use reqwest::Client;

use crate::{IbkrError, Result};

use crate::oauth::cache::payload::CachedLiveSession;
use crate::oauth::cache::redis::{cache_lock_value, RedisLiveSessionCache};
use crate::oauth::cache::{memory, LiveSessionCache};
use crate::oauth::OAuthConfig;

use super::LiveSession;

#[derive(Clone)]
pub struct LiveSessionProvider {
    http: Client,
    config: OAuthConfig,
    cache: LiveSessionCache,
}

impl LiveSessionProvider {
    pub fn new(http: Client, config: OAuthConfig) -> Result<Self> {
        let cache = LiveSessionCache::from_config(&config.live_session_cache, &config)?;
        Ok(Self {
            http,
            config,
            cache,
        })
    }

    pub async fn get_or_refresh(&self) -> Result<LiveSession> {
        match &self.cache {
            LiveSessionCache::Disabled => LiveSession::request(&self.http, &self.config).await,
            LiveSessionCache::Memory(state) => {
                memory::get_or_refresh(state, &self.http, &self.config).await
            }
            LiveSessionCache::Redis(cache) => self.get_or_refresh_redis(cache).await,
        }
    }

    async fn get_or_refresh_redis(&self, cache: &RedisLiveSessionCache) -> Result<LiveSession> {
        if let Some(cached) = cache
            .read_valid(self.config.live_session_cache.refresh_skew)
            .await?
        {
            return Ok(cached.into());
        }

        let lock_value = cache_lock_value();
        if cache.try_lock(&lock_value).await? {
            let result = self.refresh_redis_locked(cache).await;
            let unlock_result = cache.unlock(&lock_value).await;
            if let Err(err) = result {
                if let Err(unlock_err) = unlock_result {
                    return Err(IbkrError::LiveSessionCache(format!(
                        "{err}; additionally failed to release refresh lock: {unlock_err}"
                    )));
                }
                return Err(err);
            }
            unlock_result?;
            return result;
        }

        cache
            .wait_for_valid(self.config.live_session_cache.refresh_skew)
            .await
    }

    async fn refresh_redis_locked(&self, cache: &RedisLiveSessionCache) -> Result<LiveSession> {
        if let Some(cached) = cache
            .read_valid(self.config.live_session_cache.refresh_skew)
            .await?
        {
            return Ok(cached.into());
        }

        let session = LiveSession::request(&self.http, &self.config).await?;
        cache
            .write(
                &CachedLiveSession::from_session(&session),
                self.config.live_session_cache.refresh_skew,
            )
            .await?;
        Ok(session)
    }
}
