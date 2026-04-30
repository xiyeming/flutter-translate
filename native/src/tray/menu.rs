pub struct TrayMenu {
    items: Vec<MenuItem>,
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub icon_name: Option<String>,
}

impl TrayMenu {
    pub fn new() -> Self {
        let items = vec![
            MenuItem {
                id: "show_window".to_string(),
                label: "显示窗口".to_string(),
                enabled: true,
                icon_name: Some("window-new".to_string()),
            },
            MenuItem {
                id: "ocr_translate".to_string(),
                label: "截图翻译".to_string(),
                enabled: true,
                icon_name: Some("camera-photo".to_string()),
            },
            MenuItem {
                id: "separator_1".to_string(),
                label: String::new(),
                enabled: false,
                icon_name: None,
            },
            MenuItem {
                id: "settings".to_string(),
                label: "设置".to_string(),
                enabled: true,
                icon_name: Some("configure".to_string()),
            },
            MenuItem {
                id: "separator_2".to_string(),
                label: String::new(),
                enabled: false,
                icon_name: None,
            },
            MenuItem {
                id: "quit".to_string(),
                label: "退出".to_string(),
                enabled: true,
                icon_name: Some("application-exit".to_string()),
            },
        ];

        Self { items }
    }

    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    pub fn action_from_index(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|item| item.id.as_str())
    }
}
