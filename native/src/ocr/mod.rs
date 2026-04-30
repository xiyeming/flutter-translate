pub mod tesseract;
pub mod screenshot;
pub mod kde;
pub mod hyprland;

use crate::ffi::error::OcrError;
use crate::ffi::types::OcrResult;
use crate::config::ConfigManager;
use image::imageops::{self, FilterType};
use image::DynamicImage;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

pub struct OcrService;

impl OcrService {
    pub fn new() -> Result<Self, OcrError> {
        Ok(Self)
    }

    pub async fn screenshot(&self) -> Result<Vec<u8>, OcrError> {
        let desktop_env = ConfigManager::detect_desktop_env();
        screenshot::DesktopScreenshot::capture(&desktop_env)
    }

    pub async fn recognize(&self, image_data: &[u8], lang: &str) -> Result<OcrResult, OcrError> {
        if image_data.is_empty() {
            return Err(OcrError::NoTextDetected);
        }

        let data = image_data.to_vec();
        let lang = lang.to_string();
        tokio::task::spawn_blocking(move || Self::recognize_blocking(&data, &lang)).await
            .map_err(|e| OcrError::CommandError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?
    }

    fn recognize_blocking(image_data: &[u8], lang: &str) -> Result<OcrResult, OcrError> {
        let start = Instant::now();

        let img = image::load_from_memory(image_data)
            .map_err(|e| OcrError::CommandError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let processed = Self::preprocess(img);

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("xym_ft_ocr.png");
        {
            let mut file = std::fs::File::create(&temp_path)
                .map_err(|e| OcrError::IoError(e))?;
            processed.write_to(&mut file, image::ImageFormat::Png)
                .map_err(|e| OcrError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            file.write_all(b"")?;
        }

        let tesseract_lang = match lang {
            "zh" | "chi" | "chi_sim" => "chi_sim+eng",
            "ja" | "jpn" => "jpn+eng",
            "ko" | "kor" => "kor+eng",
            _ => "eng",
        };

        let psm = match lang {
            "zh" | "chi" | "chi_sim" => "6",
            "ja" | "jpn" | "ko" | "kor" => "6",
            _ => "3",
        };

        let output = Command::new("tesseract")
            .arg(temp_path.to_str().unwrap())
            .arg("stdout")
            .arg("-l").arg(tesseract_lang)
            .arg("--psm").arg(psm)
            .arg("-c").arg("tessedit_write_images=false")
            .output()
            .map_err(|e| OcrError::CommandError(e))?;

        std::fs::remove_file(&temp_path).ok();

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let elapsed = start.elapsed().as_millis() as u64;

        Ok(OcrResult {
            text,
            confidence: 0.0,
            language: lang.to_string(),
            processing_time_ms: elapsed,
        })
    }

    /// 图像预处理管道：灰度 → 对比度增强 → 2x 放大 → 二值化
    fn preprocess(img: DynamicImage) -> DynamicImage {
        let gray = DynamicImage::ImageLuma8(img.grayscale().to_luma8());
        let contrasted = DynamicImage::ImageLuma8(imageops::contrast(&gray.to_luma8(), 30.0));
        let enlarged = contrasted.resize_exact(
            contrasted.width() * 2,
            contrasted.height() * 2,
            FilterType::Lanczos3,
        );
        DynamicImage::ImageLuma8(imageops::contrast(&enlarged.to_luma8(), 20.0))
    }
}

static OCR_SERVICE: once_cell::sync::Lazy<tokio::sync::Mutex<OcrService>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(OcrService::new().unwrap()));

pub async fn get_ocr_service() -> tokio::sync::MutexGuard<'static, OcrService> {
    OCR_SERVICE.lock().await
}
