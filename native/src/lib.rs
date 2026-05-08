#![allow(unexpected_cfgs)]

mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

pub mod ffi;
pub mod platform;
pub mod translate;
pub mod config;
pub mod ocr;
pub mod update;

#[cfg(test)]
mod tests;

use std::sync::Once;
use std::path::PathBuf;
use tracing_subscriber::prelude::*;

static INIT: Once = Once::new();

static RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
    once_cell::sync::Lazy::new(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    });

fn log_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("xym-ft/logs")
}

fn init_file_logging() {
    let log_path = log_dir();
    std::fs::create_dir_all(&log_path).ok();

    let file_appender = tracing_appender::rolling::daily(&log_path, "waylex");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // 故意泄漏 guard，让 appender 在程序生命周期内持续运行
    Box::leak(Box::new(_guard));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking);

    let env_filter = tracing_subscriber::EnvFilter::from_default_env();

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init();
}

fn cleanup_old_logs() {
    let log_path = log_dir();
    let cutoff = std::time::SystemTime::now()
        - std::time::Duration::from_secs(15 * 24 * 60 * 60);

    if let Ok(entries) = std::fs::read_dir(&log_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("log") {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified < cutoff {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
}

/// 初始化所有服务（托盘、翻译引擎、配置管理等）
/// 在 Flutter 应用启动时调用
pub fn init_services() {
    INIT.call_once(|| {
        init_file_logging();
        cleanup_old_logs();
        tracing::info!("Initializing flutter-translate services...");

        if let Err(e) = crate::platform::init_platform() {
            tracing::warn!("Platform init failed (maybe already initialized): {}", e);
        }

        RUNTIME.spawn(async {
            crate::translate::init_router().await;
            tracing::info!("Translation router initialized");
        });

        tracing::info!("Services initialized");
    });
}
