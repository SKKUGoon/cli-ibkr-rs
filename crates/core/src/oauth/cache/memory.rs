use std::sync::Arc;

use reqwest::Client;
use tokio::sync::Mutex;

use crate::Result;

use super::payload::{now_ms, CachedLiveSession};
use crate::oauth::{LiveSession, OAuthConfig};

pub(in crate::oauth) type MemoryLiveSessionCache = Arc<Mutex<Option<CachedLiveSession>>>;

pub(in crate::oauth) async fn get_or_refresh(
    state: &MemoryLiveSessionCache,
    http: &Client,
    config: &OAuthConfig,
) -> Result<LiveSession> {
    let mut guard = state.lock().await;
    if let Some(cached) = guard.as_ref() {
        if cached.is_valid(config.live_session_cache.refresh_skew, now_ms()) {
            return Ok(cached.clone().into());
        }
    }

    let session = LiveSession::request(http, config).await?;
    *guard = Some(CachedLiveSession::from_session(&session));
    Ok(session)
}
