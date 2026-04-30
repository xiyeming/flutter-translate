pub mod storage;
pub mod secret;
pub mod desktop_env;

use crate::ffi::error::ConfigError;
use crate::ffi::types::{
    ProviderConfig, ActiveSession, ShortcutBinding, LanguagePref, UserConfig, DesktopEnv,
    PromptTemplate,
};
use chrono::Utc;
use sqlx::{FromRow, Row};

pub struct ConfigManager;

impl ConfigManager {
    pub async fn init() -> Result<(), ConfigError> {
        storage::get_pool().await;
        Ok(())
    }

    pub fn detect_desktop_env() -> DesktopEnv {
        let xdg_current_desktop = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_lowercase();

        if xdg_current_desktop.contains("hyprland") {
            DesktopEnv::Hyprland
        } else if xdg_current_desktop.contains("kde") {
            DesktopEnv::KdePlasma
        } else if xdg_current_desktop.contains("gnome") {
            DesktopEnv::Gnome
        } else {
            DesktopEnv::Unknown
        }
    }

    // ========== 厂商配置 ==========

    pub async fn get_all_providers() -> Result<Vec<ProviderConfig>, ConfigError> {
        let pool = storage::get_pool().await;

        #[derive(FromRow)]
        struct ProviderRow {
            id: String,
            name: String,
            api_url: Option<String>,
            model: String,
            auth_type: String,
            is_active: i64,
            sort_order: i64,
            system_prompt: Option<String>,
            created_at: String,
        }

        let rows = sqlx::query_as::<_, ProviderRow>(
            "SELECT id, name, api_url, model, auth_type, is_active, sort_order, system_prompt, created_at 
             FROM providers ORDER BY sort_order ASC"
        ).fetch_all(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        let mut providers = Vec::new();
        for row in rows {
            let api_key = secret::get_api_key(&row.id).await.ok();
            providers.push(ProviderConfig {
                id: row.id,
                name: row.name,
                api_key,
                api_url: row.api_url,
                model: row.model,
                auth_type: row.auth_type,
                is_active: row.is_active != 0,
                sort_order: row.sort_order as i32,
                system_prompt: row.system_prompt,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                    .unwrap_or_else(|_| chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
                    .with_timezone(&Utc),
            });
        }

        Ok(providers)
    }

    pub async fn save_provider(config: ProviderConfig) -> Result<(), ConfigError> {
        let pool = storage::get_pool().await;

        if let Some(api_key) = &config.api_key {
            let _ = secret::set_api_key(&config.id, api_key).await;
        }

        sqlx::query(
            r#"
            INSERT INTO providers (id, name, api_url, model, auth_type, is_active, sort_order, system_prompt, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, datetime('now'), datetime('now'))
            ON CONFLICT(id) DO UPDATE SET
                name = $2, api_url = $3, model = $4, auth_type = $5,
                is_active = $6, sort_order = $7, system_prompt = $8, updated_at = datetime('now')
            "#
        )
        .bind(&config.id)
        .bind(&config.name)
        .bind(&config.api_url)
        .bind(&config.model)
        .bind(&config.auth_type)
        .bind(if config.is_active { 1 } else { 0 })
        .bind(config.sort_order)
        .bind(&config.system_prompt)
        .execute(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(())
    }

    pub async fn delete_provider(id: &str) -> Result<(), ConfigError> {
        let pool = storage::get_pool().await;

        secret::delete_api_key(id).await.ok();

        sqlx::query("DELETE FROM providers WHERE id = $1")
            .bind(id)
            .execute(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(())
    }

    // ========== 会话管理 ==========

    pub async fn get_active_session() -> Result<ActiveSession, ConfigError> {
        let pool = storage::get_pool().await;

        let row = sqlx::query(
            "SELECT last_provider_id, last_compare_providers, last_used FROM active_sessions WHERE id = 1"
        ).fetch_one(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        let compare_providers: Vec<String> = serde_json::from_str(
            &row.try_get::<String, _>("last_compare_providers").unwrap_or_default()
        ).unwrap_or_default();

        let last_used_str: Option<String> = row.try_get("last_used").ok();
        let last_used = last_used_str
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now());

        Ok(ActiveSession {
            last_provider_id: row.try_get::<String, _>("last_provider_id").unwrap_or_default(),
            last_compare_providers: compare_providers,
            last_used,
        })
    }

    pub async fn update_session(
        provider_id: Option<String>,
        compare_providers: Option<Vec<String>>,
    ) -> Result<(), ConfigError> {
        let pool = storage::get_pool().await;

        let session = Self::get_active_session().await?;
        let new_provider_id = provider_id.unwrap_or(session.last_provider_id);
        let new_compare_providers = compare_providers.unwrap_or(session.last_compare_providers);
        let compare_json = serde_json::to_string(&new_compare_providers)
            .map_err(|e| ConfigError::SerdeError(e))?;

        sqlx::query(
            "UPDATE active_sessions SET last_provider_id = $1, last_compare_providers = $2, last_used = datetime('now') WHERE id = 1"
        )
        .bind(&new_provider_id)
        .bind(&compare_json)
        .execute(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(())
    }

    // ========== 快捷键配置 ==========

    pub async fn get_all_shortcuts() -> Result<Vec<ShortcutBinding>, ConfigError> {
        let pool = storage::get_pool().await;

        #[derive(FromRow)]
        struct ShortcutRow {
            id: String,
            action: String,
            key_combination: String,
            enabled: i64,
        }

        let rows = sqlx::query_as::<_, ShortcutRow>(
            "SELECT id, action, key_combination, enabled FROM shortcut_bindings"
        ).fetch_all(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(rows.into_iter().map(|row| ShortcutBinding {
            id: row.id,
            action: row.action,
            key_combination: row.key_combination,
            enabled: row.enabled != 0,
        }).collect())
    }

    pub async fn save_shortcut(binding: ShortcutBinding) -> Result<(), ConfigError> {
        let pool = storage::get_pool().await;

        sqlx::query(
            r#"
            INSERT INTO shortcut_bindings (id, action, key_combination, enabled)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(id) DO UPDATE SET
                action = $2, key_combination = $3, enabled = $4
            "#
        )
        .bind(&binding.id)
        .bind(&binding.action)
        .bind(&binding.key_combination)
        .bind(if binding.enabled { 1 } else { 0 })
        .execute(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(())
    }

    // ========== 语言偏好 ==========

    pub async fn get_language_prefs() -> Result<Vec<LanguagePref>, ConfigError> {
        let pool = storage::get_pool().await;

        #[derive(FromRow)]
        struct LangRow {
            code: String,
            display_name: String,
            usage_count: i64,
            is_favorite: i64,
        }

        let rows = sqlx::query_as::<_, LangRow>(
            "SELECT code, display_name, usage_count, is_favorite FROM language_prefs ORDER BY usage_count DESC"
        ).fetch_all(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(rows.into_iter().map(|row| LanguagePref {
            code: row.code,
            display_name: row.display_name,
            usage_count: row.usage_count as i32,
            is_favorite: row.is_favorite != 0,
        }).collect())
    }

    // ========== 用户配置 ==========

    pub async fn get_user_config() -> Result<UserConfig, ConfigError> {
        let pool = storage::get_pool().await;

        let row = sqlx::query(
            "SELECT id, theme, default_target_lang, auto_detect, history_enabled, created_at, updated_at FROM user_config WHERE id = 'default'"
        ).fetch_one(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        let created_at_str: Option<String> = row.try_get::<String, _>("created_at").ok();
        let updated_at_str: Option<String> = row.try_get::<String, _>("updated_at").ok();

        Ok(UserConfig {
            id: row.try_get::<String, _>("id").unwrap_or_default(),
            theme: row.try_get::<String, _>("theme").unwrap_or_else(|_| "system".to_string()),
            default_target_lang: row.try_get::<String, _>("default_target_lang").unwrap_or_else(|_| "zh".to_string()),
            auto_detect: row.try_get::<i64, _>("auto_detect").map(|v| v != 0).unwrap_or(true),
            history_enabled: row.try_get::<i64, _>("history_enabled").map(|v| v != 0).unwrap_or(false),
            created_at: created_at_str
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
            updated_at: updated_at_str
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|| Utc::now()),
        })
    }

    // ========== 提示词模板 ==========

    pub async fn get_all_prompt_templates() -> Result<Vec<PromptTemplate>, ConfigError> {
        let pool = storage::get_pool().await;

        #[derive(FromRow)]
        struct TemplateRow {
            id: String,
            name: String,
            content: String,
            is_active: i64,
            created_at: String,
        }

        let rows = sqlx::query_as::<_, TemplateRow>(
            "SELECT id, name, content, is_active, created_at FROM prompt_templates ORDER BY created_at DESC"
        ).fetch_all(&pool).await.map_err(|e| ConfigError::DbError(e))?;

        Ok(rows.into_iter().map(|r| PromptTemplate {
            id: r.id,
            name: r.name,
            content: r.content,
            is_active: r.is_active != 0,
            created_at: chrono::DateTime::parse_from_rfc3339(&r.created_at)
                .unwrap_or_else(|_| chrono::DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z").unwrap())
                .with_timezone(&Utc),
        }).collect())
    }

    pub async fn save_prompt_template(tpl: PromptTemplate) -> Result<(), ConfigError> {
        let pool = storage::get_pool().await;
        if tpl.is_active {
            sqlx::query("UPDATE prompt_templates SET is_active = 0")
                .execute(&pool).await.map_err(|e| ConfigError::DbError(e))?;
        }
        sqlx::query(
            r#"INSERT INTO prompt_templates (id, name, content, is_active, created_at)
               VALUES ($1, $2, $3, $4, datetime('now'))
               ON CONFLICT(id) DO UPDATE SET name = $2, content = $3, is_active = $4"#
        )
        .bind(&tpl.id).bind(&tpl.name).bind(&tpl.content)
        .bind(if tpl.is_active { 1 } else { 0 })
        .execute(&pool).await.map_err(|e| ConfigError::DbError(e))?;
        Ok(())
    }

    pub async fn delete_prompt_template(id: &str) -> Result<(), ConfigError> {
        let pool = storage::get_pool().await;
        sqlx::query("DELETE FROM prompt_templates WHERE id = $1")
            .bind(id).execute(&pool).await.map_err(|e| ConfigError::DbError(e))?;
        Ok(())
    }

    pub async fn get_active_prompt() -> Result<Option<PromptTemplate>, ConfigError> {
        let pool = storage::get_pool().await;
        let row = sqlx::query("SELECT id, name, content, is_active, created_at FROM prompt_templates WHERE is_active = 1")
            .fetch_optional(&pool).await.map_err(|e| ConfigError::DbError(e))?;
        match row {
            Some(r) => {
                let created_at: String = r.get("created_at");
                Ok(Some(PromptTemplate {
                    id: r.get("id"), name: r.get("name"), content: r.get("content"),
                    is_active: r.get::<i64, _>("is_active") != 0,
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now),
                }))
            }
            None => Ok(None),
        }
    }
}
