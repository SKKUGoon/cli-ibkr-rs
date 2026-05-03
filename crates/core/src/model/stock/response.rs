use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockLookupResult {
    pub symbol: String,
    pub name_: Option<String>,
    pub conid: i64,
    pub exchange: Option<String>,
}
