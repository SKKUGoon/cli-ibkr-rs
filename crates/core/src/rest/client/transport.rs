use reqwest::header::CONTENT_LENGTH;
use reqwest::Method;
use serde::Serialize;
use serde_json::Value;

use crate::rest::{request, response};
use crate::Result;

use super::IbkrClient;

impl IbkrClient {
    pub(super) async fn get_json(&self, path: &str, params: &[(String, String)]) -> Result<Value> {
        self.request_json(Method::GET, path, params, None::<&()>)
            .await
    }

    pub(super) async fn post_json<T: Serialize + ?Sized>(
        &self,
        path: &str,
        params: &[(String, String)],
        body: Option<&T>,
    ) -> Result<Value> {
        self.request_json(Method::POST, path, params, body).await
    }

    pub(super) async fn delete_json(
        &self,
        path: &str,
        params: &[(String, String)],
    ) -> Result<Value> {
        self.request_json(Method::DELETE, path, params, None::<&()>)
            .await
    }

    pub(super) async fn request_json<T: Serialize + ?Sized>(
        &self,
        method: Method,
        path: &str,
        params: &[(String, String)],
        body: Option<&T>,
    ) -> Result<Value> {
        let session = self.live_sessions.get_or_refresh().await?;
        let url = self.config.endpoint_url(path);
        let method_name = method.as_str().to_string();
        let headers =
            request::signed_headers(&method_name, &url, params, &session.token, &self.config)?;
        let mut request = self.http.request(method, &url).query(params);
        for (key, value) in headers {
            request = request.header(key, value);
        }
        if let Some(body) = body {
            request = request.json(body);
        } else if method_name == "POST" {
            request = request.header(CONTENT_LENGTH, "0").body(Vec::new());
        }
        response::parse_json_response(request.send().await?).await
    }
}
