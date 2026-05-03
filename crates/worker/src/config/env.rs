use std::env;
use std::path::PathBuf;

use super::RawConfig;
use crate::error::WorkerError;

pub fn apply(config: &mut RawConfig) -> Result<(), WorkerError> {
    set_string(&mut config.base_url, "IBKR_BASE_URL");
    set_string(&mut config.consumer_key, "IBKR_CONSUMER_KEY");
    set_string(&mut config.realm, "IBKR_REALM");
    set_string(&mut config.access_token, "IBKR_ACCESS_TOKEN");
    set_string(&mut config.access_token_secret, "IBKR_ACCESS_TOKEN_SECRET");
    set_path(&mut config.signature_key_path, "IBKR_SIGNATURE_KEY_PATH");
    set_path(&mut config.encryption_key_path, "IBKR_ENCRYPTION_KEY_PATH");
    set_path(&mut config.dh_param_path, "IBKR_DH_PARAM_PATH");
    set_u64(&mut config.timeout_seconds, "IBKR_TIMEOUT_SECONDS")?;
    set_string(&mut config.lst_cache_mode, "IBKR_LST_CACHE_MODE");
    set_string(&mut config.redis_url, "IBKR_REDIS_URL");
    set_string(&mut config.redis_key_prefix, "IBKR_REDIS_KEY_PREFIX");
    set_u64(
        &mut config.lst_refresh_skew_seconds,
        "IBKR_LST_REFRESH_SKEW_SECONDS",
    )?;
    set_u64(
        &mut config.lst_lock_ttl_seconds,
        "IBKR_LST_LOCK_TTL_SECONDS",
    )?;
    Ok(())
}

fn set_string(target: &mut Option<String>, key: &str) {
    if let Ok(value) = env::var(key) {
        *target = Some(value);
    }
}

fn set_path(target: &mut Option<PathBuf>, key: &str) {
    if let Ok(value) = env::var(key) {
        *target = Some(PathBuf::from(value));
    }
}

fn set_u64(target: &mut Option<u64>, key: &'static str) -> Result<(), WorkerError> {
    if let Ok(value) = env::var(key) {
        let parsed = value.parse().map_err(|source| WorkerError::InvalidEnv {
            key,
            value: value.clone(),
            source,
        })?;
        *target = Some(parsed);
    }
    Ok(())
}
