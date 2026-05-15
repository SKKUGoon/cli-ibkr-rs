use std::env;
use std::path::PathBuf;

use super::RawConfig;
use crate::error::WorkerError;

// Abstracts `std::env::var` so config loading can be exercised in tests
// without touching the process environment. `std::env::set_var` is
// process-global and race-prone in parallel test runs; routing through a
// trait lets unit tests pass in an in-memory map instead.
pub trait EnvSource {
    fn get(&self, key: &str) -> Option<String>;
}

pub struct ProcessEnv;

impl EnvSource for ProcessEnv {
    fn get(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

pub fn apply(config: &mut RawConfig, env: &dyn EnvSource) -> Result<(), WorkerError> {
    set_string(&mut config.base_url, "IBKR_BASE_URL", env);
    set_string(&mut config.consumer_key, "IBKR_CONSUMER_KEY", env);
    set_string(&mut config.realm, "IBKR_REALM", env);
    set_string(&mut config.access_token, "IBKR_ACCESS_TOKEN", env);
    set_string(
        &mut config.access_token_secret,
        "IBKR_ACCESS_TOKEN_SECRET",
        env,
    );
    set_path(
        &mut config.signature_key_path,
        "IBKR_SIGNATURE_KEY_PATH",
        env,
    );
    set_path(
        &mut config.encryption_key_path,
        "IBKR_ENCRYPTION_KEY_PATH",
        env,
    );
    set_path(&mut config.dh_param_path, "IBKR_DH_PARAM_PATH", env);
    set_u64(&mut config.timeout_seconds, "IBKR_TIMEOUT_SECONDS", env)?;
    set_string(&mut config.lst_cache_mode, "IBKR_LST_CACHE_MODE", env);
    set_string(&mut config.redis_url, "IBKR_REDIS_URL", env);
    set_string(&mut config.redis_key_prefix, "IBKR_REDIS_KEY_PREFIX", env);
    set_u64(
        &mut config.lst_refresh_skew_seconds,
        "IBKR_LST_REFRESH_SKEW_SECONDS",
        env,
    )?;
    set_u64(
        &mut config.lst_lock_ttl_seconds,
        "IBKR_LST_LOCK_TTL_SECONDS",
        env,
    )?;
    set_string(&mut config.database, "IBKR_DATABASE", env);
    Ok(())
}

fn set_string(target: &mut Option<String>, key: &str, env: &dyn EnvSource) {
    if let Some(value) = env.get(key) {
        *target = Some(value);
    }
}

fn set_path(target: &mut Option<PathBuf>, key: &str, env: &dyn EnvSource) {
    if let Some(value) = env.get(key) {
        *target = Some(PathBuf::from(value));
    }
}

fn set_u64(
    target: &mut Option<u64>,
    key: &'static str,
    env: &dyn EnvSource,
) -> Result<(), WorkerError> {
    if let Some(value) = env.get(key) {
        let parsed = value.parse().map_err(|source| WorkerError::InvalidEnv {
            key,
            value: value.clone(),
            source,
        })?;
        *target = Some(parsed);
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::EnvSource;
    use std::collections::HashMap;

    pub struct MapEnv(pub HashMap<String, String>);

    impl MapEnv {
        pub fn new() -> Self {
            Self(HashMap::new())
        }

        pub fn with(mut self, key: &str, value: &str) -> Self {
            self.0.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl EnvSource for MapEnv {
        fn get(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::MapEnv;
    use super::*;

    #[test]
    fn apply_only_overwrites_keys_that_are_present() {
        let env = MapEnv::new().with("IBKR_CONSUMER_KEY", "from-env");
        let mut config = RawConfig {
            consumer_key: Some("from-file".to_string()),
            realm: Some("from-file".to_string()),
            ..RawConfig::default()
        };

        apply(&mut config, &env).unwrap();

        // Present env var overrides the file-supplied value.
        assert_eq!(config.consumer_key.as_deref(), Some("from-env"));
        // Absent env var leaves the file-supplied value intact.
        assert_eq!(config.realm.as_deref(), Some("from-file"));
    }

    #[test]
    fn apply_parses_numeric_envs() {
        let env = MapEnv::new()
            .with("IBKR_TIMEOUT_SECONDS", "45")
            .with("IBKR_LST_REFRESH_SKEW_SECONDS", "90")
            .with("IBKR_LST_LOCK_TTL_SECONDS", "20");
        let mut config = RawConfig::default();

        apply(&mut config, &env).unwrap();

        assert_eq!(config.timeout_seconds, Some(45));
        assert_eq!(config.lst_refresh_skew_seconds, Some(90));
        assert_eq!(config.lst_lock_ttl_seconds, Some(20));
    }

    #[test]
    fn apply_reports_invalid_numeric_env_with_key_and_value() {
        let env = MapEnv::new().with("IBKR_TIMEOUT_SECONDS", "not-a-number");
        let mut config = RawConfig::default();

        let err = apply(&mut config, &env).unwrap_err();

        match err {
            WorkerError::InvalidEnv { key, value, .. } => {
                assert_eq!(key, "IBKR_TIMEOUT_SECONDS");
                assert_eq!(value, "not-a-number");
            }
            other => panic!("expected InvalidEnv, got {other:?}"),
        }
    }

    #[test]
    fn apply_converts_path_strings_to_pathbuf() {
        let env = MapEnv::new().with("IBKR_DH_PARAM_PATH", "/tmp/dhparam.pem");
        let mut config = RawConfig::default();

        apply(&mut config, &env).unwrap();

        assert_eq!(
            config.dh_param_path.as_deref(),
            Some(std::path::Path::new("/tmp/dhparam.pem"))
        );
    }
}
