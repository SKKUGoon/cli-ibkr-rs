use std::collections::BTreeMap;

use super::percent;

pub fn build(
    method: &str,
    url: &str,
    oauth_params: &BTreeMap<String, String>,
    request_params: &[(String, String)],
    prepend: Option<&str>,
) -> String {
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
