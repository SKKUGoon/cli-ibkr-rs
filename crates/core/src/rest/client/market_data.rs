use serde_json::Value;

use crate::model::history::HistoryRequest;
use crate::model::stock::{self, StockConidRequest, StockLookupResult};
use crate::rest::endpoint;
use crate::Result;

use super::IbkrClient;

impl IbkrClient {
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
}
