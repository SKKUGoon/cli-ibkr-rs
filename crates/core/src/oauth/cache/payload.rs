use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::oauth::LiveSession;
use crate::{IbkrError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(in crate::oauth) struct CachedLiveSession {
    pub(in crate::oauth) token: String,
    pub(in crate::oauth) expiration_ms: i64,
    pub(in crate::oauth) created_at_ms: i64,
}

impl CachedLiveSession {
    pub(in crate::oauth) fn from_session(session: &LiveSession) -> Self {
        Self {
            token: session.token.clone(),
            expiration_ms: session.expiration_ms,
            created_at_ms: now_ms(),
        }
    }

    pub(in crate::oauth) fn is_valid(&self, refresh_skew: Duration, now_ms: i64) -> bool {
        !self.token.trim().is_empty()
            && self.expiration_ms.saturating_sub(duration_ms(refresh_skew)) > now_ms
    }
}

impl From<CachedLiveSession> for LiveSession {
    fn from(value: CachedLiveSession) -> Self {
        Self {
            token: value.token,
            expiration_ms: value.expiration_ms,
        }
    }
}

pub(in crate::oauth) fn decode_cached_payload(raw: &str) -> Option<CachedLiveSession> {
    serde_json::from_str::<CachedLiveSession>(raw).ok()
}

pub(in crate::oauth) fn ttl_seconds(expiration_ms: i64, refresh_skew: Duration) -> Result<u64> {
    let ttl_ms = expiration_ms
        .saturating_sub(duration_ms(refresh_skew))
        .saturating_sub(now_ms());
    if ttl_ms <= 0 {
        return Err(IbkrError::LiveSessionCache(
            "refusing to cache expired live session token".to_string(),
        ));
    }
    Ok(u64::try_from((ttl_ms + 999) / 1000).unwrap_or(1).max(1))
}

pub(in crate::oauth) fn now_ms() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    i64::try_from(millis).unwrap_or(i64::MAX)
}

fn duration_ms(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}
