use zbus::Connection;
use crate::ffi::error::TrayError;

#[allow(dead_code)]
pub struct TrayIndicator {
    connection: Option<Connection>,
    tooltip: String,
    icon_name: String,
}

impl TrayIndicator {
    pub fn new() -> Self {
        Self {
            connection: None,
            tooltip: "Flutter Translate".to_string(),
            icon_name: "accessories-dictionary".to_string(),
        }
    }

    pub async fn register(&mut self) -> Result<(), TrayError> {
        let conn = Connection::session().await
            .map_err(|e| TrayError::DbusError(e))?;

        let pid = std::process::id();
        let bus_name = format!("org.kde.StatusNotifierItem-{}-1", pid);
        let service_name: &str = &bus_name;

        conn.request_name(service_name)
            .await
            .map_err(|e| TrayError::DbusError(e))?;

        self.connection = Some(conn);
        tracing::info!("SNI tray registered: {}", bus_name);
        Ok(())
    }

    pub fn is_registered(&self) -> bool {
        self.connection.is_some()
    }

    pub async fn update_tooltip(&mut self, tooltip: &str) -> Result<(), TrayError> {
        self.tooltip = tooltip.to_string();
        Ok(())
    }
}

impl Drop for TrayIndicator {
    fn drop(&mut self) {
        tracing::info!("Tray indicator dropped");
    }
}
