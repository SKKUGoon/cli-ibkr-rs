use serde::de::DeserializeOwned;

use crate::{IbkrError, Result};

// Identifies which IBKR-side request a response belongs to, so non-2xx
// errors surface enough context for an operator to tell which call failed
// without leaking signed Authorization headers or token secrets.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RequestContext<'a> {
    /// Stage of the OAuth flow this request is part of. Two callers exist
    /// today: `"live-session-token"` (LST refresh) and
    /// `"protected-resource"` (any signed API call once an LST is held).
    pub phase: &'static str,
    pub method: &'a str,
    pub url: &'a str,
}

pub(crate) async fn parse_json_response<T: DeserializeOwned>(
    response: reqwest::Response,
    context: RequestContext<'_>,
) -> Result<T> {
    let status = response.status();
    let body = response.text().await?;
    parse_json_body(status.as_u16(), status.is_success(), &body, context)
}

fn parse_json_body<T: DeserializeOwned>(
    status: u16,
    success: bool,
    body: &str,
    context: RequestContext<'_>,
) -> Result<T> {
    if !success {
        return Err(IbkrError::HttpStatus {
            phase: context.phase,
            method: context.method.to_string(),
            url: context.url.to_string(),
            status,
            body: body.to_string(),
        });
    }
    Ok(serde_json::from_str(body)?)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct TestBody {
        ok: bool,
    }

    fn lst_context() -> RequestContext<'static> {
        RequestContext {
            phase: "live-session-token",
            method: "POST",
            url: "https://api.ibkr.com/v1/api/oauth/live_session_token",
        }
    }

    fn protected_context() -> RequestContext<'static> {
        RequestContext {
            phase: "protected-resource",
            method: "POST",
            url: "https://api.ibkr.com/v1/api/iserver/auth/ssodh/init",
        }
    }

    #[test]
    fn success_response_parses_json() {
        let body: TestBody = parse_json_body(200, true, r#"{"ok":true}"#, lst_context()).unwrap();
        assert_eq!(body, TestBody { ok: true });
    }

    #[test]
    fn non_success_response_preserves_status_body_method_url_phase() {
        let err = parse_json_body::<TestBody>(
            410,
            false,
            r#"{"error":"invalid consumer"}"#,
            protected_context(),
        )
        .expect_err("non-success status should be returned as an HTTP status error");

        match err {
            IbkrError::HttpStatus {
                phase,
                method,
                url,
                status,
                body,
            } => {
                assert_eq!(phase, "protected-resource");
                assert_eq!(method, "POST");
                assert_eq!(url, "https://api.ibkr.com/v1/api/iserver/auth/ssodh/init");
                assert_eq!(status, 410);
                assert_eq!(body, r#"{"error":"invalid consumer"}"#);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn display_includes_phase_endpoint_and_body_but_no_auth_headers() {
        let err = parse_json_body::<TestBody>(410, false, r#"{"detail":"gone"}"#, lst_context())
            .expect_err("non-success status should be returned as an HTTP status error");

        let display = err.to_string();
        assert!(display.contains("410"), "missing status: {display}");
        assert!(
            display.contains("live-session-token"),
            "missing phase: {display}"
        );
        assert!(
            display.contains("POST https://api.ibkr.com/v1/api/oauth/live_session_token"),
            "missing method+url: {display}"
        );
        assert!(
            display.contains(r#"{"detail":"gone"}"#),
            "missing body: {display}"
        );
        // Sensitive material that must never appear in the rendered error.
        assert!(
            !display.to_ascii_lowercase().contains("authorization"),
            "Authorization header must not leak: {display}"
        );
        assert!(
            !display.to_ascii_lowercase().contains("oauth_signature"),
            "OAuth signature must not leak: {display}"
        );
    }

    #[test]
    fn invalid_success_json_returns_json_error() {
        let err = parse_json_body::<TestBody>(200, true, "not json", lst_context())
            .expect_err("invalid success body should fail JSON parsing");
        assert!(matches!(err, IbkrError::Json(_)));
    }
}
