use crate::ffi::error::HotkeyError;
use crate::ffi::types::ShortcutBinding;
use zbus::Connection;
use std::collections::HashMap;

const KGLOBALACCEL_SERVICE: &str = "org.kde.kglobalaccel";
const KGLOBALACCEL_PATH: &str = "/kglobalaccel";
const KGLOBALACCEL_IFACE: &str = "org.kde.KGlobalAccel";

pub struct KdeHotkeyService {
    shortcuts: HashMap<String, ShortcutBinding>,
}

impl KdeHotkeyService {
    pub fn new() -> Self {
        Self {
            shortcuts: HashMap::new(),
        }
    }

    pub async fn register_all(&mut self, shortcuts: Vec<ShortcutBinding>) -> Result<(), HotkeyError> {
        let conn = Connection::session().await
            .map_err(|e| HotkeyError::DbusError(e))?;

        let proxy = zbus::Proxy::new(
            &conn,
            KGLOBALACCEL_SERVICE,
            KGLOBALACCEL_PATH,
            KGLOBALACCEL_IFACE,
        )
        .await
        .map_err(|e| HotkeyError::DbusError(e))?;

        for binding in &shortcuts {
            if !binding.enabled {
                continue;
            }
            self.register_single(&proxy, binding).await?;
        }

        self.shortcuts.clear();
        for s in shortcuts {
            self.shortcuts.insert(s.action.clone(), s);
        }

        Ok(())
    }

    async fn register_single(
        &self,
        proxy: &zbus::Proxy<'_>,
        binding: &ShortcutBinding,
    ) -> Result<(), HotkeyError> {
        let action = binding.action.clone();
        let keys = binding.key_combination.clone();

        let _reply = proxy
            .call_method(
                "registerShortcut",
                &(action.clone(), keys.clone(), keys),
            )
            .await
            .map_err(|e| HotkeyError::DbusError(e))?;

        tracing::info!(
            "Registered KDE shortcut: {} -> {}",
            binding.key_combination,
            action
        );
        Ok(())
    }
}
