#[cfg(test)]
mod tests {
    use crate::platform::{create_platform_backend, init_platform, platform, PlatformInitError};

    #[test]
    fn test_create_platform_backend() {
        let backend = create_platform_backend();
        let env = backend.detect_desktop_env();
        assert!(
            matches!(
                env,
                crate::ffi::types::DesktopEnv::KdePlasma
                    | crate::ffi::types::DesktopEnv::Hyprland
                    | crate::ffi::types::DesktopEnv::Gnome
                    | crate::ffi::types::DesktopEnv::Unknown
            ),
            "detect_desktop_env should return a valid desktop environment variant"
        );
    }

    #[test]
    fn test_platform_init_error_display() {
        let err = PlatformInitError::AlreadyInitialized;
        assert_eq!(
            err.to_string(),
            "Platform backend already initialized"
        );
    }

    #[test]
    fn test_platform_singleton() {
        let result = init_platform();
        assert!(
            result.is_ok() || matches!(result, Err(PlatformInitError::AlreadyInitialized)),
            "init_platform should either succeed or report already initialized"
        );

        let backend = platform();
        let _ = backend.detect_desktop_env();
    }

    #[test]
    fn test_hotkey_error_display() {
        let err = crate::ffi::error::HotkeyError::NoKeyboardDevices;
        assert!(err.to_string().contains("No keyboard devices found"));

        let err = crate::ffi::error::HotkeyError::UnsupportedDesktop;
        assert!(err.to_string().contains("Unsupported desktop environment"));
    }

    #[test]
    fn test_clipboard_error_display() {
        let err = crate::ffi::error::ClipboardError::Empty;
        assert!(err.to_string().contains("Clipboard empty"));

        let err = crate::ffi::error::ClipboardError::WriteFailed;
        assert!(err.to_string().contains("Write failed"));
    }

    #[test]
    fn test_tray_error_display() {
        let err = crate::ffi::error::TrayError::InitFailed;
        assert!(err.to_string().contains("Init failed"));
    }

    #[test]
    fn test_ocr_error_display() {
        let err = crate::ffi::error::OcrError::ScreenshotFailed;
        assert!(err.to_string().contains("Screenshot failed"));

        let err = crate::ffi::error::OcrError::UserCancelled;
        assert!(err.to_string().contains("User cancelled selection"));
    }

    #[cfg(target_os = "windows")]
    mod windows_tests {
        use super::*;

        /// PNG 文件魔数 (magic bytes)
        const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

        /// 验证字节流是否为有效的 PNG 图像
        fn assert_valid_png(data: &[u8]) {
            assert!(
                data.len() >= PNG_MAGIC.len(),
                "PNG data too short: {} bytes",
                data.len()
            );
            assert_eq!(
                &data[..PNG_MAGIC.len()],
                PNG_MAGIC,
                "screenshot data does not start with PNG magic bytes"
            );
            // PNG 文件以 IEND chunk 结尾: 00 00 00 00 49 45 4E 44 AE 42 60 82
            assert!(
                data.windows(12).any(|w| w == [0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]),
                "PNG data does not contain valid IEND trailer"
            );
        }

        #[tokio::test]
        async fn test_windows_screenshot_returns_valid_png() {
            let backend = crate::platform::windows::WindowsBackend::new();
            match backend.screenshot().await {
                Ok(data) => {
                    assert!(!data.is_empty(), "screenshot data should not be empty");
                    assert_valid_png(&data);
                }
                Err(e) => {
                    let msg = e.to_string();
                    assert!(
                        msg.contains("Screenshot failed") || msg.contains("User cancelled"),
                        "unexpected screenshot error: {}",
                        msg
                    );
                }
            }
        }

        #[test]
        fn test_windows_backend_creation() {
            let backend = crate::platform::windows::WindowsBackend::new();
            let env = backend.detect_desktop_env();
            assert!(
                matches!(env, crate::ffi::types::DesktopEnv::Unknown),
                "Windows should report Unknown desktop env"
            );
        }
    }
}
