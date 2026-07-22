use serde_json::Value;

use crate::rest::endpoint;
use crate::Result;

use super::IbkrClient;

impl IbkrClient {
    pub async fn accounts(&self) -> Result<Value> {
        self.portfolio_accounts().await
    }

    pub async fn portfolio_accounts(&self) -> Result<Value> {
        self.get_json(endpoint::PORTFOLIO_ACCOUNTS, &[]).await
    }

    pub async fn brokerage_accounts(&self) -> Result<Value> {
        self.get_json(endpoint::BROKERAGE_ACCOUNTS, &[]).await
    }

    pub async fn account_profit_and_loss(&self) -> Result<Value> {
        self.get_json(endpoint::ACCOUNT_PROFIT_AND_LOSS, &[]).await
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

    pub async fn live_positions(
        &self,
        account_id: &str,
        model: Option<&str>,
        sort: Option<&str>,
        direction: Option<&str>,
    ) -> Result<Value> {
        let mut query_parameters = Vec::new();
        push_optional_query_parameter(&mut query_parameters, "model", model);
        push_optional_query_parameter(&mut query_parameters, "sort", sort);
        push_optional_query_parameter(&mut query_parameters, "direction", direction);
        self.get_json(&endpoint::live_positions(account_id), &query_parameters)
            .await
    }

    pub async fn trades(&self, account_id: Option<&str>, days: Option<u8>) -> Result<Value> {
        let mut params = Vec::new();
        if let Some(account_id) = account_id {
            params.push(("accountId".to_string(), account_id.to_string()));
        }
        if let Some(days) = days {
            params.push(("days".to_string(), days.to_string()));
        }
        self.get_json(endpoint::TRADES, &params).await
    }

    pub async fn live_orders(&self, account_id: Option<&str>, force: bool) -> Result<Value> {
        let mut params = vec![("force".to_string(), force.to_string())];
        if let Some(account_id) = account_id {
            params.push(("accountId".to_string(), account_id.to_string()));
        }
        self.get_json(endpoint::LIVE_ORDERS, &params).await
    }
}

fn push_optional_query_parameter(
    query_parameters: &mut Vec<(String, String)>,
    key: &str,
    value: Option<&str>,
) {
    if let Some(value) = value {
        query_parameters.push((key.to_string(), value.to_string()));
    }
}
