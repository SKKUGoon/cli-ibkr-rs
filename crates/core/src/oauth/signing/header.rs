use std::collections::BTreeMap;

pub type OAuthParams = BTreeMap<String, String>;

pub fn authorization_header(realm: &str, params: &OAuthParams) -> String {
    let mut all = params.clone();
    all.insert("realm".to_string(), realm.to_string());
    let pairs = all
        .iter()
        .map(|(key, value)| format!("{key}=\"{value}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("OAuth {pairs}")
}

pub fn standard_headers(auth_header: String) -> Vec<(&'static str, String)> {
    vec![
        ("Accept", "*/*".to_string()),
        ("Accept-Encoding", "gzip,deflate".to_string()),
        ("Authorization", auth_header),
        ("Connection", "keep-alive".to_string()),
        ("Host", "api.ibkr.com".to_string()),
        ("User-Agent", "ibkrctl/0.1".to_string()),
    ]
}
