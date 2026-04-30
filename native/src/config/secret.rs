use crate::ffi::error::ConfigError;
use sqlx::Row;

const SERVICE_NAME: &str = "xym-ft";

/// Always read from SQLite first (reliable on all platforms).
/// Fall back to keyring for backwards compatibility.
pub async fn get_api_key(provider_id: &str) -> Result<String, ConfigError> {
    match get_from_sqlite(provider_id).await {
        Ok(key) => Ok(key),
        Err(_) => get_from_keyring(provider_id),
    }
}

/// Always write to SQLite (primary).
/// Try keyring as best-effort bonus — failure is ignored.
pub async fn set_api_key(provider_id: &str, api_key: &str) -> Result<(), ConfigError> {
    set_to_sqlite(provider_id, api_key).await?;
    let _ = set_to_keyring(provider_id, api_key);
    Ok(())
}

pub async fn delete_api_key(provider_id: &str) -> Result<(), ConfigError> {
    let _ = delete_from_keyring(provider_id);
    delete_from_sqlite(provider_id).await
}

fn get_from_keyring(provider_id: &str) -> Result<String, ConfigError> {
    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
        .map_err(|e| ConfigError::KeyNotFound(e.to_string()))?;
    entry.get_password()
        .map_err(|_| ConfigError::KeyNotFound("keyring entry not found".to_string()))
}

fn set_to_keyring(provider_id: &str, api_key: &str) -> Result<(), ConfigError> {
    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
        .map_err(|e| ConfigError::KeyNotFound(e.to_string()))?;
    entry.set_password(api_key)
        .map_err(|e| ConfigError::KeyNotFound(e.to_string()))
}

fn delete_from_keyring(provider_id: &str) -> Result<(), ConfigError> {
    let entry = keyring::Entry::new(SERVICE_NAME, provider_id)
        .map_err(|e| ConfigError::KeyNotFound(e.to_string()))?;
    entry.delete_credential()
        .map_err(|e| ConfigError::KeyNotFound(e.to_string()))
}

async fn get_from_sqlite(provider_id: &str) -> Result<String, ConfigError> {
    let pool = crate::config::storage::get_pool().await;
    let row = sqlx::query("SELECT api_key FROM provider_keys WHERE provider_id = $1")
        .bind(provider_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| ConfigError::DbError(e))?;

    match row {
        Some(row) => {
            let key: String = row.get("api_key");
            Ok(key)
        }
        None => Err(ConfigError::KeyNotFound("api key not found".to_string())),
    }
}

async fn set_to_sqlite(provider_id: &str, api_key: &str) -> Result<(), ConfigError> {
    let pool = crate::config::storage::get_pool().await;
    sqlx::query(
        r#"INSERT INTO provider_keys (provider_id, api_key) VALUES ($1, $2)
           ON CONFLICT(provider_id) DO UPDATE SET api_key = $2"#
    )
    .bind(provider_id)
    .bind(api_key)
    .execute(&pool)
    .await
    .map_err(|e| ConfigError::DbError(e))?;
    Ok(())
}

async fn delete_from_sqlite(provider_id: &str) -> Result<(), ConfigError> {
    let pool = crate::config::storage::get_pool().await;
    sqlx::query("DELETE FROM provider_keys WHERE provider_id = $1")
        .bind(provider_id)
        .execute(&pool)
        .await
        .map_err(|e| ConfigError::DbError(e))?;
    Ok(())
}
