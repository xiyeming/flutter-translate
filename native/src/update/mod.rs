use crate::ffi::error::ConfigError;
use crate::ffi::types::UpdateInfo;
use serde::Deserialize;
use sqlx::Row;

const GITHUB_OWNER: &str = "XiYeMing";
const GITHUB_REPO: &str = "Waylex";
const GITHUB_API_URL: &str = "https://api.github.com";

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    body: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

pub struct UpdateService;

impl UpdateService {
    pub async fn check_update(current_version: String) -> Result<Option<UpdateInfo>, ConfigError> {
        let skipped = Self::get_skipped_version().await?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent(format!("Waylex/{} (Rust; GitHub API)", current_version))
            .build()
            .map_err(|e| ConfigError::SecretError(e.to_string()))?;

        let url = format!("{}/repos/{}/{}/releases/latest", GITHUB_API_URL, GITHUB_OWNER, GITHUB_REPO);
        let response = client.get(&url).send().await
            .map_err(|e| ConfigError::SecretError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ConfigError::SecretError(
                format!("GitHub API error: {}", response.status())
            ));
        }

        let release: GithubRelease = response.json().await
            .map_err(|e| ConfigError::SecretError(e.to_string()))?;

        let latest_version = release.tag_name.trim_start_matches('v').to_string();

        if latest_version == skipped {
            tracing::info!("Latest version {} skipped by user", latest_version);
            Self::update_last_check_time().await?;
            return Ok(None);
        }

        let needs_update = Self::version_is_newer(&current_version, &latest_version)?;
        if !needs_update {
            tracing::info!("Current version {} is up to date (latest: {})", current_version, latest_version);
            Self::update_last_check_time().await?;
            return Ok(None);
        }

        let download_url = Self::find_download_url(&release.assets)
            .unwrap_or_else(|| release.html_url.clone());

        let release_notes = release.body.unwrap_or_default();

        Ok(Some(UpdateInfo {
            current_version,
            latest_version,
            download_url,
            release_notes,
        }))
    }

    pub async fn set_skipped_version(version: String) -> Result<(), ConfigError> {
        let pool = crate::config::storage::get_pool().await;
        sqlx::query(
            "UPDATE app_update SET skipped_version = $1 WHERE id = 'update_state'"
        )
        .bind(&version)
        .execute(&pool).await.map_err(ConfigError::DbError)?;
        Ok(())
    }

    pub async fn get_skipped_version() -> Result<String, ConfigError> {
        let pool = crate::config::storage::get_pool().await;
        let row = sqlx::query("SELECT skipped_version FROM app_update WHERE id = 'update_state'")
            .fetch_one(&pool).await.map_err(ConfigError::DbError)?;
        Ok(row.try_get::<String, _>("skipped_version").unwrap_or_default())
    }

    pub async fn should_check() -> Result<bool, ConfigError> {
        let pool = crate::config::storage::get_pool().await;
        let row = sqlx::query("SELECT last_check_at FROM app_update WHERE id = 'update_state'")
            .fetch_one(&pool).await.map_err(ConfigError::DbError)?;

        let last_check: String = row.try_get("last_check_at").unwrap_or_default();
        let last = chrono::DateTime::parse_from_rfc3339(&last_check)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let now = chrono::Utc::now();
        let should = match last {
            Some(dt) => now.signed_duration_since(dt).num_hours() >= 24,
            None => true,
        };
        Ok(should)
    }

    async fn update_last_check_time() -> Result<(), ConfigError> {
        let pool = crate::config::storage::get_pool().await;
        sqlx::query(
            "UPDATE app_update SET last_check_at = datetime('now') WHERE id = 'update_state'"
        )
        .execute(&pool).await.map_err(ConfigError::DbError)?;
        Ok(())
    }

    fn version_is_newer(current: &str, latest: &str) -> Result<bool, ConfigError> {
        let current_parsed = semver::Version::parse(current.trim_start_matches('v'))
            .map_err(|e| ConfigError::ValidationError {
                field: "current_version".to_string(),
                message: e.to_string(),
            })?;
        let latest_parsed = semver::Version::parse(latest.trim_start_matches('v'))
            .map_err(|e| ConfigError::ValidationError {
                field: "latest_version".to_string(),
                message: e.to_string(),
            })?;
        Ok(latest_parsed > current_parsed)
    }

    fn find_download_url(assets: &[GithubAsset]) -> Option<String> {
        let platform_suffix = Self::platform_asset_suffix();
        assets.iter()
            .find(|a| a.name.ends_with(&platform_suffix))
            .map(|a| a.browser_download_url.clone())
    }

    fn platform_asset_suffix() -> String {
        #[cfg(target_os = "linux")]
        {
            if cfg!(target_arch = "aarch64") {
                "linux-aarch64.tar.gz".to_string()
            } else {
                "linux-x86_64.tar.gz".to_string()
            }
        }
        #[cfg(target_os = "macos")]
        {
            "macos-universal.zip".to_string()
        }
        #[cfg(target_os = "windows")]
        {
            "windows-x64.zip".to_string()
        }
    }
}
