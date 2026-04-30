use crate::ffi::types::DesktopEnv;
use std::process::Command;

/// 桌面环境适配器 - 根据检测到的桌面环境自动选择实现
pub struct DesktopAdapter {
    env: DesktopEnv,
}

impl DesktopAdapter {
    pub fn new(env: DesktopEnv) -> Self {
        Self { env }
    }

    pub fn env(&self) -> &DesktopEnv {
        &self.env
    }

    pub fn is_kde(&self) -> bool {
        matches!(self.env, DesktopEnv::KdePlasma)
    }

    pub fn is_hyprland(&self) -> bool {
        matches!(self.env, DesktopEnv::Hyprland)
    }

    /// 检测 D-Bus 服务是否可用
    pub fn check_dbus_service(service: &str) -> bool {
        let output = Command::new("dbus-send")
            .args(["--session", "--print-reply", "--dest=org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus.NameHasOwner", &format!("string:{}", service)])
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains("true")
            }
            Err(_) => false,
        }
    }

    /// 检查 KDE KGlobalAccel 服务是否可用
    pub fn has_kde_global_accel() -> bool {
        Self::check_dbus_service("org.kde.kglobalaccel")
    }

    /// 检查是否安装了 wl-copy/wl-paste
    pub fn has_wl_clipboard() -> bool {
        let has_copy = Command::new("which")
            .arg("wl-copy")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let has_paste = Command::new("which")
            .arg("wl-paste")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        has_copy && has_paste
    }

    /// 检查是否安装了 grim + slurp (Hyprland 截图工具)
    pub fn has_grim_slurp() -> bool {
        let has_grim = Command::new("which")
            .arg("grim")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let has_slurp = Command::new("which")
            .arg("slurp")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        has_grim && has_slurp
    }

    /// 检查 Waybar 是否运行
    pub fn is_waybar_running() -> bool {
        Command::new("pgrep")
            .arg("waybar")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// 获取截图命令
    pub fn screenshot_cmd(&self) -> Option<String> {
        match self.env {
            DesktopEnv::Hyprland => {
                if Self::has_grim_slurp() {
                    Some("grim -g \"$(slurp)\" -".to_string())
                } else {
                    None
                }
            }
            DesktopEnv::KdePlasma | DesktopEnv::Gnome | DesktopEnv::Unknown => None,
        }
    }

    /// 获取快捷键实现类型
    pub fn hotkey_backend(&self) -> HotkeyBackend {
        match self.env {
            DesktopEnv::KdePlasma => {
                if Self::has_kde_global_accel() {
                    HotkeyBackend::KdeGlobalAccel
                } else {
                    HotkeyBackend::Evdev
                }
            }
            DesktopEnv::Hyprland | DesktopEnv::Gnome | DesktopEnv::Unknown => {
                HotkeyBackend::Evdev
            }
        }
    }

    /// 获取剪贴板实现类型
    pub fn clipboard_backend(&self) -> ClipboardBackend {
        if Self::has_wl_clipboard() {
            ClipboardBackend::WlClipboard
        } else {
            ClipboardBackend::None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyBackend {
    KdeGlobalAccel,
    Evdev,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardBackend {
    WlClipboard,
    None,
}
