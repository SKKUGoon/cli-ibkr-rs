use std::path::PathBuf;
use std::time::Duration;

use crate::{IbkrError, Result};

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub base_url: String,
    pub consumer_key: String,
    pub realm: String,
    pub access_token: String,
    pub access_token_secret: String,
    pub signature_key_path: PathBuf,
    pub encryption_key_path: PathBuf,
    pub dh_param_path: PathBuf,
    pub timeout: Duration,
    pub live_session_cache: LiveSessionCacheConfig,
}

impl OAuthConfig {
    pub fn validate(&self) -> Result<()> {
        required("consumer_key", &self.consumer_key)?;
        required("realm", &self.realm)?;
        required("access_token", &self.access_token)?;
        required("access_token_secret", &self.access_token_secret)?;
        Ok(())
    }

    pub fn endpoint_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveSessionCacheMode {
    Redis,
    Memory,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct LiveSessionCacheConfig {
    pub mode: LiveSessionCacheMode,
    pub redis_url: Option<String>,
    pub key_prefix: String,
    pub refresh_skew: Duration,
    pub lock_ttl: Duration,
}

impl Default for LiveSessionCacheConfig {
    fn default() -> Self {
        Self {
            mode: LiveSessionCacheMode::Memory,
            redis_url: None,
            key_prefix: "ibkr:oauth:lst".to_string(),
            refresh_skew: Duration::from_secs(60),
            lock_ttl: Duration::from_secs(15),
        }
    }
}

fn required(name: &'static str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(IbkrError::MissingConfig(name));
    }
    Ok(())
}
