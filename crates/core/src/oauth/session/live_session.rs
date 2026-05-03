use reqwest::header::CONTENT_LENGTH;
use reqwest::Client;
use serde::Deserialize;

use crate::rest::response;
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
struct LiveSessionResponse {
    diffie_hellman_response: String,
    live_session_token_signature: String,
    live_session_token_expiration: i64,
}

impl LiveSession {
    pub async fn request(client: &Client, config: &OAuthConfig) -> Result<Self> {
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

        let response = send_lst_request(client, &url, config, &params).await?;
        let token = session_token::compute(
            &dh.prime,
            &dh_random,
            &response.diffie_hellman_response,
            &prepend,
        )?;
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

async fn send_lst_request(
    client: &Client,
    url: &str,
    config: &OAuthConfig,
    params: &OAuthParams,
) -> Result<LiveSessionResponse> {
    let auth = header::authorization_header(&config.realm, params);
    let mut request = client
        .post(url)
        .header(CONTENT_LENGTH, "0")
        .body(Vec::new());
    for (key, value) in header::standard_headers(auth) {
        request = request.header(key, value);
    }
    let response = request.send().await?;
    response::parse_json_response(response).await
}
