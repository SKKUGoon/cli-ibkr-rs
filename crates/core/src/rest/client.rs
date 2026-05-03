use reqwest::header::CONTENT_LENGTH;
use reqwest::{Client, Method};
use serde::Serialize;
use serde_json::{json, Value};

use crate::model::history::HistoryRequest;
use crate::model::order::{OrderRequest, PlaceOrdersRequest, ReplyRequest};
use crate::model::stock::{self, StockConidRequest, StockLookupResult};
use crate::oauth::{LiveSessionProvider, OAuthConfig};
use crate::Result;

use super::{endpoint, request, response};

pub struct IbkrClient {
    http: Client,
    config: OAuthConfig,
    live_sessions: LiveSessionProvider,
}

impl IbkrClient {
    pub fn new(config: OAuthConfig) -> Result<Self> {
        let http = Client::builder().timeout(config.timeout).build()?;
        let live_sessions = LiveSessionProvider::new(http.clone(), config.clone())?;
        Ok(Self {
            http,
            config,
            live_sessions,
        })
    }

    pub async fn auth_status(&self) -> Result<Value> {
        self.post_json(endpoint::AUTH_STATUS, &[], None::<&()>)
            .await
    }

    pub async fn init_session(&self, compete: bool) -> Result<Value> {
        let body = json!({ "publish": true, "compete": compete });
        self.post_json(endpoint::INIT_SESSION, &[], Some(&body))
            .await
    }

    pub async fn fetch_history(&self, request: &HistoryRequest) -> Result<Value> {
        self.get_json(endpoint::HISTORY, &request.params()).await
    }

    pub async fn stock_conid(&self, request: &StockConidRequest) -> Result<Value> {
        let response = self.get_json(endpoint::STOCKS, &request.params()).await?;
        stock::conid_by_symbol(&response, request)
    }

    pub async fn stock_lookup(&self, request: &StockConidRequest) -> Result<StockLookupResult> {
        let response = self.get_json(endpoint::STOCKS, &request.params()).await?;
        stock::stock_by_symbol(&response, request)
    }

    pub async fn accounts(&self) -> Result<Value> {
        self.get_json(endpoint::ACCOUNTS, &[]).await
    }

    pub async fn account_summary(&self, account_id: &str) -> Result<Value> {
        self.get_json(&endpoint::account_summary(account_id), &[])
            .await
    }

    pub async fn portfolio_summary(&self, account_id: &str) -> Result<Value> {
        self.get_json(&endpoint::portfolio_summary(account_id), &[])
            .await
    }

    pub async fn ledger(&self, account_id: &str) -> Result<Value> {
        self.get_json(&endpoint::ledger(account_id), &[]).await
    }

    pub async fn positions(&self, account_id: &str, page: u32) -> Result<Value> {
        self.get_json(&endpoint::positions(account_id, page), &[])
            .await
    }

    pub async fn live_orders(&self, account_id: Option<&str>, force: bool) -> Result<Value> {
        let mut params = vec![("force".to_string(), force.to_string())];
        if let Some(account_id) = account_id {
            params.push(("accountId".to_string(), account_id.to_string()));
        }
        self.get_json(endpoint::LIVE_ORDERS, &params).await
    }

    pub async fn place_orders(
        &self,
        account_id: &str,
        request: &PlaceOrdersRequest,
    ) -> Result<Value> {
        self.post_json(
            &endpoint::order::place_orders(account_id),
            &[],
            Some(request),
        )
        .await
    }

    pub async fn whatif_order(
        &self,
        account_id: &str,
        request: &PlaceOrdersRequest,
    ) -> Result<Value> {
        self.post_json(
            &endpoint::order::whatif_orders(account_id),
            &[],
            Some(request),
        )
        .await
    }

    pub async fn reply(&self, reply_id: &str, confirmed: bool) -> Result<Value> {
        let body = ReplyRequest { confirmed };
        self.post_json(&endpoint::order::reply(reply_id), &[], Some(&body))
            .await
    }

    pub async fn cancel_order(&self, account_id: &str, order_id: &str) -> Result<Value> {
        self.delete_json(&endpoint::order::cancel_order(account_id, order_id), &[])
            .await
    }

    pub async fn modify_order(
        &self,
        account_id: &str,
        order_id: &str,
        request: &OrderRequest,
    ) -> Result<Value> {
        self.post_json(
            &endpoint::order::modify_order(account_id, order_id),
            &[],
            Some(request),
        )
        .await
    }

    pub async fn order_status(&self, order_id: &str) -> Result<Value> {
        self.get_json(&endpoint::order::order_status(order_id), &[])
            .await
    }

    pub async fn order_algos(
        &self,
        conid: &str,
        algos: &[String],
        add_description: bool,
        add_params: bool,
    ) -> Result<Value> {
        let mut params = Vec::new();
        if !algos.is_empty() {
            params.push(("algos".to_string(), algos.join(";")));
        }
        params.push((
            "addDescription".to_string(),
            bool_query_value(add_description).to_string(),
        ));
        params.push((
            "addParams".to_string(),
            bool_query_value(add_params).to_string(),
        ));

        self.get_json(&endpoint::order::algos(conid), &params).await
    }

    async fn get_json(&self, path: &str, params: &[(String, String)]) -> Result<Value> {
        self.request_json(Method::GET, path, params, None::<&()>)
            .await
    }

    async fn post_json<T: Serialize + ?Sized>(
        &self,
        path: &str,
        params: &[(String, String)],
        body: Option<&T>,
    ) -> Result<Value> {
        self.request_json(Method::POST, path, params, body).await
    }

    async fn delete_json(&self, path: &str, params: &[(String, String)]) -> Result<Value> {
        self.request_json(Method::DELETE, path, params, None::<&()>)
            .await
    }

    async fn request_json<T: Serialize + ?Sized>(
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

fn bool_query_value(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}
