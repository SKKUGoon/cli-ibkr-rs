use std::future::Future;

use reqwest::header::CONTENT_LENGTH;
use reqwest::Client;
use serde::Deserialize;

use crate::rest::response::{self, RequestContext};
use crate::Result;

use super::session_token;
use crate::oauth::materials::{dh, keys};
use crate::oauth::signing::{base_string, header, hmac, nonce};
use crate::oauth::{OAuthConfig, OAuthParams};

#[derive(Debug, Clone)]
pub struct LiveSession {
    pub token: String,
    pub expiration_ms: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LiveSessionResponse {
    pub(crate) diffie_hellman_response: String,
    pub(crate) live_session_token_signature: String,
    pub(crate) live_session_token_expiration: i64,
}

// Separates the HTTP round-trip from the LST crypto orchestration so the
// latter can be exercised in tests with a fake transport. Production code
// uses `ReqwestTransport` which wraps a `reqwest::Client`.
pub(crate) trait LstTransport {
    fn post_lst(
        &self,
        url: &str,
        auth_header: String,
    ) -> impl Future<Output = Result<LiveSessionResponse>> + Send;
}

pub(crate) struct ReqwestTransport<'a>(pub &'a Client);

impl LstTransport for ReqwestTransport<'_> {
    async fn post_lst(&self, url: &str, auth_header: String) -> Result<LiveSessionResponse> {
        // CONTENT_LENGTH: 0 with an empty body is required — IBKR rejects POSTs
        // to /oauth/live_session_token that omit it.
        let mut request = self
            .0
            .post(url)
            .header(CONTENT_LENGTH, "0")
            .body(Vec::new());
        for (key, value) in header::standard_headers(auth_header) {
            request = request.header(key, value);
        }
        let context = RequestContext {
            phase: "live-session-token",
            method: "POST",
            url,
        };
        response::parse_json_response(request.send().await?, context).await
    }
}

impl LiveSession {
    pub async fn request(client: &Client, config: &OAuthConfig) -> Result<Self> {
        Self::request_with_transport(&ReqwestTransport(client), config).await
    }

    pub(crate) async fn request_with_transport<T: LstTransport>(
        transport: &T,
        config: &OAuthConfig,
    ) -> Result<Self> {
        config.validate()?;
        let dh = dh::read_dh_params(&config.dh_param_path)?;

        // IBKR requires a fresh 256-bit Diffie-Hellman private value for each
        // live session token request. It is never logged or persisted.
        let dh_random = session_token::random_256_bit();
        let challenge = dh.generator.modpow(&dh_random, &dh.prime).to_str_radix(16);

        // The prepend is the decrypted access token secret. It prefixes only
        // the LST base string and must not be reused for normal API requests.
        let prepend = decrypted_prepend(config)?;
        let url = config.endpoint_url("oauth/live_session_token");

        let mut params = oauth_params(config, challenge);
        let base = base_string::build("POST", &url, &params, &[], Some(&prepend));

        // LST requests use RSA-SHA256. Protected resource requests use the
        // computed LST with HMAC-SHA256 instead.
        let key = keys::read_private_key(&config.signature_key_path)?;
        params.insert("oauth_signature".to_string(), hmac::rsa_sha256(&base, key));

        let auth_header = header::authorization_header(&config.realm, &params);
        let response = transport.post_lst(&url, auth_header).await?;

        let token = session_token::compute(
            &dh.prime,
            &dh_random,
            &response.diffie_hellman_response,
            &prepend,
        )?;
        // Cross-check: IBKR returns HMAC-SHA1_hex(LST, consumer_key) so both
        // sides can confirm they derived identical session keys.
        session_token::validate(&token, &response.live_session_token_signature, config)?;

        Ok(Self {
            token,
            expiration_ms: response.live_session_token_expiration,
        })
    }
}

fn oauth_params(config: &OAuthConfig, challenge: String) -> OAuthParams {
    OAuthParams::from([
        (
            "oauth_consumer_key".to_string(),
            config.consumer_key.clone(),
        ),
        ("oauth_nonce".to_string(), nonce::nonce_hex(16)),
        (
            "oauth_signature_method".to_string(),
            "RSA-SHA256".to_string(),
        ),
        ("oauth_timestamp".to_string(), nonce::timestamp_seconds()),
        ("oauth_token".to_string(), config.access_token.clone()),
        ("diffie_hellman_challenge".to_string(), challenge),
    ])
}

fn decrypted_prepend(config: &OAuthConfig) -> Result<String> {
    let key = keys::read_private_key(&config.encryption_key_path)?;
    hmac::decrypt_prepend(&config.access_token_secret, key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IbkrError;
    use std::path::PathBuf;
    use std::time::Duration;

    use crate::oauth::config::LiveSessionCacheConfig;

    fn config_missing_consumer_key() -> OAuthConfig {
        OAuthConfig {
            base_url: "https://example.test".to_string(),
            consumer_key: String::new(),
            realm: "limited_poa".to_string(),
            access_token: "tok".to_string(),
            access_token_secret: "secret".to_string(),
            signature_key_path: PathBuf::from("/nonexistent/signature.pem"),
            encryption_key_path: PathBuf::from("/nonexistent/encryption.pem"),
            dh_param_path: PathBuf::from("/nonexistent/dhparam.pem"),
            timeout: Duration::from_secs(5),
            live_session_cache: LiveSessionCacheConfig::default(),
        }
    }

    struct UnreachableTransport;

    impl LstTransport for UnreachableTransport {
        async fn post_lst(&self, _url: &str, _auth_header: String) -> Result<LiveSessionResponse> {
            panic!("transport must not be invoked when config validation fails");
        }
    }

    // `request_with_transport` must short-circuit on config validation
    // before doing any file I/O or HTTP work. This locks that behavior in
    // place so a future refactor cannot accidentally reorder the checks.
    #[tokio::test]
    async fn request_short_circuits_on_invalid_config() {
        let config = config_missing_consumer_key();
        let err = LiveSession::request_with_transport(&UnreachableTransport, &config)
            .await
            .unwrap_err();
        assert!(matches!(err, IbkrError::MissingConfig(_)));
    }
}
