use crate::ffi::error::OcrError;
use std::process::Command;
use std::fs;

pub async fn capture_screenshot() -> Result<Vec<u8>, OcrError> {
    let temp_file = std::env::temp_dir().join("xym_ft_kde_screenshot.png");

    // Use spectacle for KDE screenshot
    if has_spectacle() {
        let output = Command::new("spectacle")
            .args(["-r", "-b", "-n", "-o"])
            .arg(temp_file.to_str().unwrap())
            .output()
            .map_err(|e| OcrError::CommandError(e))?;

        if !output.status.success() {
            return Err(OcrError::ScreenshotFailed);
        }

        let data = fs::read(&temp_file).map_err(|e| OcrError::IoError(e))?;
        fs::remove_file(&temp_file).ok();
        return Ok(data);
    }

    Err(OcrError::PermissionDenied)
}

fn has_spectacle() -> bool {
    Command::new("which")
        .arg("spectacle")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
