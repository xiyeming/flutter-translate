use crate::ffi::error::OcrError;
use crate::ffi::types::DesktopEnv;
use std::process::Command;
use std::fs;

pub struct DesktopScreenshot;

impl DesktopScreenshot {
    pub fn capture(env: &DesktopEnv) -> Result<Vec<u8>, OcrError> {
        match env {
            DesktopEnv::KdePlasma => Self::capture_kde(),
            DesktopEnv::Hyprland => Self::capture_hyprland(),
            DesktopEnv::Gnome => Self::capture_gnome(),
            DesktopEnv::Unknown => Err(OcrError::PermissionDenied),
        }
    }

    fn capture_kde() -> Result<Vec<u8>, OcrError> {
        // 使用 spectacle 命令行截图（需要用户交互选择区域）
        if Self::has_spectacle() {
            let temp_file = std::env::temp_dir().join("xym_ft_screenshot.png");

            let output = Command::new("spectacle")
                .args(["-r", "-b", "-n", "-o"])
                .arg(temp_file.to_str().unwrap())
                .output()
                .map_err(|e| OcrError::CommandError(e))?;

            if !output.status.success() {
                return Err(OcrError::ScreenshotFailed);
            }

            let data = fs::read(&temp_file)
                .map_err(|e| OcrError::IoError(e))?;

            fs::remove_file(&temp_file).ok();

            if data.is_empty() {
                return Err(OcrError::NoTextDetected);
            }

            return Ok(data);
        }

        // 回退到 grim（如果可用）
        if Self::has_grim() {
            return Self::capture_grim_area();
        }

        Err(OcrError::PermissionDenied)
    }

    fn capture_hyprland() -> Result<Vec<u8>, OcrError> {
        Self::capture_grim_area()
    }

    fn capture_gnome() -> Result<Vec<u8>, OcrError> {
        let temp_file = std::env::temp_dir().join("xym_ft_screenshot.png");

        let output = Command::new("gnome-screenshot")
            .args(["-a", "-f"])
            .arg(temp_file.to_str().unwrap())
            .output()
            .map_err(|e| OcrError::CommandError(e))?;

        if !output.status.success() {
            return Err(OcrError::ScreenshotFailed);
        }

        let data = fs::read(&temp_file)
            .map_err(|e| OcrError::IoError(e))?;

        fs::remove_file(&temp_file).ok();
        Ok(data)
    }

    fn capture_grim_area() -> Result<Vec<u8>, OcrError> {
        // 使用 slurp 选择区域，grim 截图到 stdout
        let slurp = Command::new("slurp")
            .arg("-d")
            .output()
            .map_err(|e| OcrError::CommandError(e))?;

        if !slurp.status.success() {
            return Err(OcrError::UserCancelled);
        }

        let area = String::from_utf8_lossy(&slurp.stdout).trim().to_string();
        if area.is_empty() {
            return Err(OcrError::UserCancelled);
        }

        let output = Command::new("grim")
            .args(["-g", &area, "-t", "png", "-"])
            .output()
            .map_err(|e| OcrError::CommandError(e))?;

        if !output.status.success() {
            return Err(OcrError::ScreenshotFailed);
        }

        Ok(output.stdout)
    }

    fn has_spectacle() -> bool {
        Command::new("which")
            .arg("spectacle")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn has_grim() -> bool {
        Command::new("which")
            .arg("grim")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
