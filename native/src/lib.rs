#![allow(unexpected_cfgs)]

mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

pub mod ffi;
pub mod tray;
pub mod hotkey;
pub mod clipboard;
pub mod ocr;
pub mod translate;
pub mod config;

#[cfg(test)]
mod tests;

use std::sync::Once;

static INIT: Once = Once::new();

static RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
    once_cell::sync::Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    });

/// 初始化所有服务（托盘、翻译引擎、配置管理等）
/// 在 Flutter 应用启动时调用
pub fn init_services() {
    INIT.call_once(|| {
        tracing::info!("Initializing flutter-translate services...");

        RUNTIME.spawn(async {
            crate::translate::init_router().await;
            tracing::info!("Translation router initialized");
        });

        tracing::info!("Services initialized");
    });
}