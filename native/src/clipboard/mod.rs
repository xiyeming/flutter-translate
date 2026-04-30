use crate::ffi::error::ClipboardError;
use std::process::Command;

pub struct ClipboardService {
    inner: WaylandClipboardService,
}

impl ClipboardService {
    pub fn new() -> Result<Self, ClipboardError> {
        Ok(Self {
            inner: WaylandClipboardService::new()?,
        })
    }

    pub fn get_text(&self) -> Result<String, ClipboardError> {
        self.inner.get_text()
    }

    pub fn set_text(&self, text: String) -> Result<(), ClipboardError> {
        self.inner.set_text(&text)
    }
}

struct WaylandClipboardService;

impl WaylandClipboardService {
    fn new() -> Result<Self, ClipboardError> {
        let has_wl_copy = Command::new("which")
            .arg("wl-copy")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let has_wl_paste = Command::new("which")
            .arg("wl-paste")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_wl_copy || !has_wl_paste {
            return Err(ClipboardError::WlError("wl-clipboard-rs not installed".into()));
        }

        Ok(Self)
    }

    fn get_text(&self) -> Result<String, ClipboardError> {
        let output = Command::new("wl-paste")
            .arg("--no-newline")
            .output()
            .map_err(|e| ClipboardError::WlError(e.to_string()))?;

        if !output.status.success() {
            return Err(ClipboardError::Empty);
        }

        String::from_utf8(output.stdout)
            .map_err(|e| ClipboardError::Utf8Error(e))
            .map(|s| s.trim().to_string())
    }

    fn set_text(&self, text: &str) -> Result<(), ClipboardError> {
        let mut child = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| ClipboardError::WlError(e.to_string()))?;

        {
            use std::io::Write;
            let stdin = child.stdin.as_mut().ok_or(ClipboardError::WriteFailed)?;
            stdin.write_all(text.as_bytes())
                .map_err(|_| ClipboardError::WriteFailed)?;
        }

        let status = child.wait()
            .map_err(|e| ClipboardError::WlError(e.to_string()))?;

        if status.success() {
            Ok(())
        } else {
            Err(ClipboardError::WriteFailed)
        }
    }
}

static CLIPBOARD_SERVICE: once_cell::sync::Lazy<ClipboardService> =
    once_cell::sync::Lazy::new(|| ClipboardService::new().unwrap());

pub fn get_clipboard_service() -> &'static ClipboardService {
    &CLIPBOARD_SERVICE
}
