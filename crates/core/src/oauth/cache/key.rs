use sha2::{Digest, Sha256};

use super::super::config::LiveSessionCacheConfig;
use super::super::OAuthConfig;

pub(super) fn cache_key(config: &LiveSessionCacheConfig, oauth: &OAuthConfig) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        config.key_prefix,
        fingerprint(&oauth.base_url),
        fingerprint(&oauth.consumer_key),
        fingerprint(&oauth.access_token),
        fingerprint(&oauth.realm)
    )
}

fn fingerprint(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    hex::encode(&digest[..16])
}
