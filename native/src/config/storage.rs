use sqlx::{Sqlite, SqlitePool, migrate::MigrateDatabase};
use std::path::PathBuf;
use crate::ffi::error::ConfigError;

const DB_NAME: &str = "xym_ft.db";

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn init() -> Result<Self, ConfigError> {
        let db_path = Self::get_db_path();
        let db_url = format!("sqlite:{}", db_path.display());

        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await
                .map_err(|e| ConfigError::DbError(e))?;
        }

        let pool = SqlitePool::connect(&db_url).await
            .map_err(|e| ConfigError::DbError(e))?;

        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn get_db_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("xym-ft");

        std::fs::create_dir_all(&config_dir).ok();
        config_dir.join(DB_NAME)
    }

    async fn run_migrations(pool: &SqlitePool) -> Result<(), ConfigError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                api_key_encrypted TEXT,
                api_url TEXT,
                model TEXT NOT NULL,
                auth_type TEXT NOT NULL DEFAULT 'api_key',
                is_active INTEGER NOT NULL DEFAULT 1,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS translation_rules (
                id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL,
                role_name TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                custom_rules TEXT NOT NULL DEFAULT '{}',
                is_default INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS active_sessions (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_provider_id TEXT NOT NULL DEFAULT '',
                last_compare_providers TEXT NOT NULL DEFAULT '[]',
                last_used TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO active_sessions (id) VALUES (1)
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS shortcut_bindings (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                key_combination TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS language_prefs (
                code TEXT PRIMARY KEY,
                display_name TEXT NOT NULL,
                usage_count INTEGER NOT NULL DEFAULT 0,
                is_favorite INTEGER NOT NULL DEFAULT 0
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_config (
                id TEXT PRIMARY KEY,
                theme TEXT NOT NULL DEFAULT 'system',
                default_target_lang TEXT NOT NULL DEFAULT 'zh',
                auto_detect INTEGER NOT NULL DEFAULT 1,
                history_enabled INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO user_config (id) VALUES ('default')
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS provider_keys (
                provider_id TEXT PRIMARY KEY,
                api_key TEXT NOT NULL
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        let _ = sqlx::query(
            "ALTER TABLE providers ADD COLUMN system_prompt TEXT"
        ).execute(pool).await;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS prompt_templates (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                content TEXT NOT NULL,
                is_active INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#
        ).execute(pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(())
    }
}

static DATABASE: once_cell::sync::Lazy<tokio::sync::Mutex<Option<Database>>> =
    once_cell::sync::Lazy::new(|| tokio::sync::Mutex::new(None));

pub async fn get_pool() -> SqlitePool {
    let mut guard = DATABASE.lock().await;
    if guard.is_none() {
        let db = Database::init().await.expect("Failed to initialize database");
        *guard = Some(db);
    }
    guard.as_ref().unwrap().pool().clone()
}
