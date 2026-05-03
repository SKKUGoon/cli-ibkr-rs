use crate::oauth::{base_string, header, hmac, nonce, OAuthConfig, OAuthParams};
use crate::Result;

pub(crate) fn signed_headers(
    method: &str,
    url: &str,
    request_params: &[(String, String)],
    token: &str,
    config: &OAuthConfig,
) -> Result<Vec<(&'static str, String)>> {
    let mut params = protected_params(config);
    let base = base_string::build(method, url, &params, request_params, None);
    params.insert(
        "oauth_signature".to_string(),
        hmac::hmac_sha256(&base, token)?,
    );
    let auth = header::authorization_header(&config.realm, &params);
    Ok(header::standard_headers(auth))
}

fn protected_params(config: &OAuthConfig) -> OAuthParams {
    OAuthParams::from([
        (
            "oauth_consumer_key".to_string(),
            config.consumer_key.clone(),
        ),
        ("oauth_nonce".to_string(), nonce::nonce_hex(16)),
        (
            "oauth_signature_method".to_string(),
            "HMAC-SHA256".to_string(),
        ),
        ("oauth_timestamp".to_string(), nonce::timestamp_seconds()),
        ("oauth_token".to_string(), config.access_token.clone()),
    ])
}
