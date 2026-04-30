use crate::ffi::error::OcrError;
use std::process::Command;

pub async fn capture_screenshot() -> Result<Vec<u8>, OcrError> {
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
