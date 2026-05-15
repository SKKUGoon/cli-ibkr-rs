use std::collections::BTreeMap;

use super::percent;

// Builds the OAuth 1.0a signature base string (RFC 5849 §3.4.1).
//
// Wire format: `METHOD&percent(URL)&percent(joined-params)` where joined-params
// are sorted ASCII-lexicographically by key. We rely on `BTreeMap` for that
// ordering — do not switch to `HashMap` without re-sorting explicitly, or
// signatures will be non-deterministic and IBKR will reject the request.
//
// For the live-session-token endpoint specifically, IBKR prepends the
// hex-encoded decrypted access token secret to the base string before signing.
// That is the `prepend` argument; pass `None` for normal protected requests.
pub fn build(
    method: &str,
    url: &str,
    oauth_params: &BTreeMap<String, String>,
    request_params: &[(String, String)],
    prepend: Option<&str>,
) -> String {
    // Merge oauth_* params with any request-specific params (query/body).
    // Both sides are already percent-encoded by their callers, so this step
    // is pure key collection — duplicate keys are not expected in IBKR flows.
    let mut params = oauth_params.clone();
    for (key, value) in request_params {
        params.insert(key.clone(), value.clone());
    }

    let param_string = params
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");

    let base = format!(
        "{}&{}&{}",
        method.to_uppercase(),
        percent::encode(url),
        percent::encode(&param_string)
    );

    match prepend {
        Some(prepend) => format!("{prepend}{base}"),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::build;
    use std::collections::BTreeMap;

    #[test]
    fn sorts_parameters() {
        let mut params = BTreeMap::new();
        params.insert("z".to_string(), "last".to_string());
        params.insert("a".to_string(), "first".to_string());
        let base = build("POST", "https://example.test/x", &params, &[], None);
        assert!(base.ends_with("a%3Dfirst%26z%3Dlast"));
    }
}
