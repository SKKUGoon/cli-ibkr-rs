mod key;
pub(super) mod memory;
pub(super) mod payload;
pub(super) mod redis;

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{IbkrError, Result};

use super::config::{LiveSessionCacheConfig, LiveSessionCacheMode};
use super::OAuthConfig;
use key::cache_key;
use memory::MemoryLiveSessionCache;
use redis::RedisLiveSessionCache;

#[derive(Clone)]
pub(super) enum LiveSessionCache {
    Disabled,
    Memory(MemoryLiveSessionCache),
    Redis(RedisLiveSessionCache),
}

impl LiveSessionCache {
    pub(super) fn from_config(
        config: &LiveSessionCacheConfig,
        oauth: &OAuthConfig,
    ) -> Result<Self> {
        match config.mode {
            LiveSessionCacheMode::Disabled => Ok(Self::Disabled),
            LiveSessionCacheMode::Memory => Ok(Self::Memory(Arc::new(Mutex::new(None)))),
            LiveSessionCacheMode::Redis => {
                let url = config
                    .redis_url
                    .as_deref()
                    .ok_or(IbkrError::MissingConfig("IBKR_REDIS_URL"))?;
                let client = ::redis::Client::open(url)?;
                Ok(Self::Redis(RedisLiveSessionCache::new(
                    client,
                    cache_key(config, oauth),
                    config.lock_ttl,
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use super::*;
    use crate::oauth::cache::payload::CachedLiveSession;
    use crate::oauth::config::{LiveSessionCacheConfig, LiveSessionCacheMode};

    fn test_config() -> OAuthConfig {
        OAuthConfig {
            base_url: "https://api.ibkr.com/v1/api".to_string(),
            consumer_key: "consumer".to_string(),
            realm: "limited_poa".to_string(),
            access_token: "access-token".to_string(),
            access_token_secret: "secret".to_string(),
            signature_key_path: PathBuf::from("signature.pem"),
            encryption_key_path: PathBuf::from("encryption.pem"),
            dh_param_path: PathBuf::from("dhparam.pem"),
            timeout: Duration::from_secs(30),
            live_session_cache: LiveSessionCacheConfig::default(),
        }
    }

    #[test]
    fn cached_token_is_valid_before_refresh_skew() {
        let now = 1_000_000;
        let cached = CachedLiveSession {
            token: "token".to_string(),
            expiration_ms: now + 120_000,
            created_at_ms: now,
        };
        assert!(cached.is_valid(Duration::from_secs(60), now));
    }

    #[test]
    fn cached_token_refreshes_within_skew() {
        let now = 1_000_000;
        let cached = CachedLiveSession {
            token: "token".to_string(),
            expiration_ms: now + 30_000,
            created_at_ms: now,
        };
        assert!(!cached.is_valid(Duration::from_secs(60), now));
    }

    #[test]
    fn cached_token_rejects_empty_token() {
        let now = 1_000_000;
        let cached = CachedLiveSession {
            token: " ".to_string(),
            expiration_ms: now + 120_000,
            created_at_ms: now,
        };
        assert!(!cached.is_valid(Duration::from_secs(60), now));
    }

    #[test]
    fn cache_key_does_not_contain_raw_identifiers() {
        let oauth = test_config();
        let config = LiveSessionCacheConfig {
            mode: LiveSessionCacheMode::Redis,
            redis_url: Some("redis://127.0.0.1/".to_string()),
            ..LiveSessionCacheConfig::default()
        };
        let key = cache_key(&config, &oauth);
        assert!(!key.contains(&oauth.consumer_key));
        assert!(!key.contains(&oauth.access_token));
        assert!(!key.contains(&oauth.realm));
        assert!(key.starts_with("ibkr:oauth:lst:"));
    }

    #[test]
    fn ttl_refuses_expired_sessions() {
        let err = payload::ttl_seconds(payload::now_ms() - 1, Duration::from_secs(60))
            .expect_err("expired token should not be cached");
        assert!(err.to_string().contains("expired live session token"));
    }

    #[test]
    fn invalid_cached_payload_is_ignored() {
        assert_eq!(payload::decode_cached_payload("not json"), None);
    }

    #[test]
    fn cache_modes_construct_without_redis_in_memory_and_disabled_modes() {
        let mut oauth = test_config();
        oauth.live_session_cache.mode = LiveSessionCacheMode::Memory;
        assert!(matches!(
            LiveSessionCache::from_config(&oauth.live_session_cache, &oauth).unwrap(),
            LiveSessionCache::Memory(_)
        ));

        oauth.live_session_cache.mode = LiveSessionCacheMode::Disabled;
        assert!(matches!(
            LiveSessionCache::from_config(&oauth.live_session_cache, &oauth).unwrap(),
            LiveSessionCache::Disabled
        ));
    }
}
