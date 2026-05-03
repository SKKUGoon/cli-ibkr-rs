#[derive(Debug, Clone)]
pub struct StockConidRequest {
    pub symbol: String,
    pub exchange: Option<String>,
    pub default_filtering: bool,
}

impl StockConidRequest {
    pub fn params(&self) -> Vec<(String, String)> {
        vec![("symbols".to_string(), self.symbol.clone())]
    }
}
