pub mod env;

use std::path::PathBuf;
use std::time::Duration;

use crate::error::WorkerError;
use ibkr_core::{LiveSessionCacheConfig, LiveSessionCacheMode, OAuthConfig};

#[derive(Debug, Default)]
pub struct RawConfig {
    pub base_url: Option<String>,
    pub consumer_key: Option<String>,
    pub realm: Option<String>,
    pub access_token: Option<String>,
    pub access_token_secret: Option<String>,
    pub signature_key_path: Option<PathBuf>,
    pub encryption_key_path: Option<PathBuf>,
    pub dh_param_path: Option<PathBuf>,
    pub timeout_seconds: Option<u64>,
    pub lst_cache_mode: Option<String>,
    pub redis_url: Option<String>,
    pub redis_key_prefix: Option<String>,
    pub lst_refresh_skew_seconds: Option<u64>,
    pub lst_lock_ttl_seconds: Option<u64>,
    pub database: Option<String>,
}

impl RawConfig {
    pub fn load(timeout_override: Option<u64>) -> Result<Self, WorkerError> {
        let mut config = Self::default();
        env::apply(&mut config)?;
        if let Some(timeout) = timeout_override {
            config.timeout_seconds = Some(timeout);
        }
        Ok(config)
    }

    pub fn oauth(self) -> Result<OAuthConfig, WorkerError> {
        Ok(OAuthConfig {
            base_url: self
                .base_url
                .unwrap_or_else(|| "https://api.ibkr.com/v1/api".to_string()),
            consumer_key: required(self.consumer_key, "IBKR_CONSUMER_KEY")?,
            realm: self.realm.unwrap_or_else(|| "limited_poa".to_string()),
            access_token: required(self.access_token, "IBKR_ACCESS_TOKEN")?,
            access_token_secret: required(self.access_token_secret, "IBKR_ACCESS_TOKEN_SECRET")?,
            signature_key_path: required(self.signature_key_path, "IBKR_SIGNATURE_KEY_PATH")?,
            encryption_key_path: required(self.encryption_key_path, "IBKR_ENCRYPTION_KEY_PATH")?,
            dh_param_path: required(self.dh_param_path, "IBKR_DH_PARAM_PATH")?,
            timeout: Duration::from_secs(self.timeout_seconds.unwrap_or(30)),
            live_session_cache: LiveSessionCacheConfig {
                mode: cache_mode(self.lst_cache_mode)?,
                redis_url: self.redis_url,
                key_prefix: self
                    .redis_key_prefix
                    .unwrap_or_else(|| "ibkr:oauth:lst".to_string()),
                refresh_skew: Duration::from_secs(self.lst_refresh_skew_seconds.unwrap_or(60)),
                lock_ttl: Duration::from_secs(self.lst_lock_ttl_seconds.unwrap_or(15)),
            },
        })
    }
}

fn required<T>(value: Option<T>, name: &'static str) -> Result<T, WorkerError> {
    value.ok_or(WorkerError::MissingConfig(name))
}

fn cache_mode(value: Option<String>) -> Result<LiveSessionCacheMode, WorkerError> {
    let value = value.unwrap_or_else(|| "memory".to_string());
    match value.trim().to_ascii_lowercase().as_str() {
        "redis" => Ok(LiveSessionCacheMode::Redis),
        "memory" => Ok(LiveSessionCacheMode::Memory),
        "disabled" => Ok(LiveSessionCacheMode::Disabled),
        _ => Err(WorkerError::InvalidCacheMode(value)),
    }
}
