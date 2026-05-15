use std::time::Duration;

use redis::AsyncCommands;
use tokio::time::{sleep, Instant};

use crate::oauth::LiveSession;
use crate::{IbkrError, Result};

use super::payload::{decode_cached_payload, now_ms, ttl_seconds, CachedLiveSession};

#[derive(Clone)]
pub(in crate::oauth) struct RedisLiveSessionCache {
    client: redis::Client,
    cache_key: String,
    lock_key: String,
    lock_ttl: Duration,
}

impl RedisLiveSessionCache {
    pub(in crate::oauth) fn new(
        client: redis::Client,
        cache_key: String,
        lock_ttl: Duration,
    ) -> Self {
        Self {
            lock_key: format!("{cache_key}:lock"),
            cache_key,
            client,
            lock_ttl,
        }
    }

    pub(in crate::oauth) async fn read_valid(
        &self,
        refresh_skew: Duration,
    ) -> Result<Option<CachedLiveSession>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let raw: Option<String> = conn.get(&self.cache_key).await?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let Some(cached) = decode_cached_payload(&raw) else {
            return Ok(None);
        };
        if cached.is_valid(refresh_skew, now_ms()) {
            Ok(Some(cached))
        } else {
            Ok(None)
        }
    }

    pub(in crate::oauth) async fn write(
        &self,
        cached: &CachedLiveSession,
        refresh_skew: Duration,
    ) -> Result<()> {
        let ttl = ttl_seconds(cached.expiration_ms, refresh_skew)?;
        let payload = serde_json::to_string(cached)?;
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let _: () = conn.set_ex(&self.cache_key, payload, ttl).await?;
        Ok(())
    }

    pub(in crate::oauth) async fn try_lock(&self, lock_value: &str) -> Result<bool> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let ttl = usize::try_from(self.lock_ttl.as_secs()).unwrap_or(usize::MAX);
        let acquired: Option<String> = redis::cmd("SET")
            .arg(&self.lock_key)
            .arg(lock_value)
            .arg("NX")
            .arg("EX")
            .arg(ttl)
            .query_async(&mut conn)
            .await?;
        Ok(acquired.as_deref() == Some("OK"))
    }

    pub(in crate::oauth) async fn unlock(&self, lock_value: &str) -> Result<()> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let current: Option<String> = conn.get(&self.lock_key).await?;
        if current.as_deref() == Some(lock_value) {
            let _: () = conn.del(&self.lock_key).await?;
        }
        Ok(())
    }

    pub(in crate::oauth) async fn wait_for_valid(
        &self,
        refresh_skew: Duration,
    ) -> Result<LiveSession> {
        let deadline = Instant::now() + self.lock_ttl + Duration::from_secs(1);
        while Instant::now() < deadline {
            sleep(Duration::from_millis(200)).await;
            if let Some(cached) = self.read_valid(refresh_skew).await? {
                return Ok(cached.into());
            }
        }
        Err(IbkrError::LiveSessionCache(
            "timed out waiting for live session token refresh lock".to_string(),
        ))
    }
}

pub(in crate::oauth) fn cache_lock_value() -> String {
    let mut bytes = [0_u8; 16];
    rand::TryRng::try_fill_bytes(&mut rand::rngs::SysRng, &mut bytes)
        .expect("OS CSPRNG unavailable");
    hex::encode(bytes)
}
