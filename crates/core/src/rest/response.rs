use serde::de::DeserializeOwned;

use crate::{IbkrError, Result};

pub(crate) async fn parse_json_response<T: DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T> {
    let status = response.status();
    let body = response.text().await?;
    parse_json_body(status.as_u16(), status.is_success(), &body)
}

fn parse_json_body<T: DeserializeOwned>(status: u16, success: bool, body: &str) -> Result<T> {
    if !success {
        return Err(IbkrError::HttpStatus {
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

    #[test]
    fn success_response_parses_json() {
        let body: TestBody = parse_json_body(200, true, r#"{"ok":true}"#).unwrap();
        assert_eq!(body, TestBody { ok: true });
    }

    #[test]
    fn non_success_response_preserves_status_and_body() {
        let err = parse_json_body::<TestBody>(401, false, r#"{"error":"invalid consumer"}"#)
            .expect_err("non-success status should be returned as an HTTP status error");

        match err {
            IbkrError::HttpStatus { status, body } => {
                assert_eq!(status, 401);
                assert_eq!(body, r#"{"error":"invalid consumer"}"#);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn invalid_success_json_returns_json_error() {
        let err = parse_json_body::<TestBody>(200, true, "not json")
            .expect_err("invalid success body should fail JSON parsing");
        assert!(matches!(err, IbkrError::Json(_)));
    }
}
