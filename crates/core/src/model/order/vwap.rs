use serde_json::json;

use crate::{IbkrError, Result};

use super::{OrderRequest, PlaceOrdersRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn as_ibkr_value(self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }

    pub fn as_client_order_id_component(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VwapOrderInput {
    pub conid: i64,
    pub side: OrderSide,
    pub quantity: f64,
    pub limit_price: f64,
    pub account_id: String,
    pub start_time: String,
    pub end_time: String,
    pub max_percent_volume: String,
    pub client_order_id: String,
}

impl VwapOrderInput {
    pub fn into_place_orders_request(self) -> Result<PlaceOrdersRequest> {
        self.validate()?;
        let order = OrderRequest {
            conid: Some(json!(self.conid)),
            side: Some(self.side.as_ibkr_value().to_string()),
            quantity: Some(json!(self.quantity)),
            order_type: Some("LMT".to_string()),
            price: Some(json!(self.limit_price)),
            acct_id: Some(self.account_id),
            tif: Some("DAY".to_string()),
            strategy: Some("Vwap".to_string()),
            strategy_parameters: Some(json!({
                "maxPctVol": self.max_percent_volume,
                "startTime": self.start_time,
                "endTime": self.end_time,
                "allowPastEndTime": "0",
                "noTakeLiq": "0",
                "speedUp": "0"
            })),
            coid: Some(self.client_order_id),
            ..OrderRequest::default()
        };
        PlaceOrdersRequest::new(vec![order])
    }

    fn validate(&self) -> Result<()> {
        if self.conid <= 0 {
            return invalid_vwap_order("conid must be positive");
        }
        if !self.quantity.is_finite() || self.quantity <= 0.0 {
            return invalid_vwap_order("quantity must be a positive finite number");
        }
        if !self.limit_price.is_finite() || self.limit_price <= 0.0 {
            return invalid_vwap_order("limit price must be a positive finite number");
        }
        let max_percent_volume = self.max_percent_volume.parse::<f64>().map_err(|_| {
            IbkrError::InvalidOrderRequest("max percent volume must be numeric".to_string())
        })?;
        if !max_percent_volume.is_finite() || max_percent_volume <= 0.0 {
            return invalid_vwap_order("max percent volume must be positive");
        }
        for (name, value) in [
            ("account id", self.account_id.as_str()),
            ("start time", self.start_time.as_str()),
            ("end time", self.end_time.as_str()),
            ("max percent volume", self.max_percent_volume.as_str()),
            ("client order id", self.client_order_id.as_str()),
        ] {
            if value.trim().is_empty() {
                return invalid_vwap_order(&format!("{name} cannot be empty"));
            }
        }
        Ok(())
    }
}

fn invalid_vwap_order<T>(message: &str) -> Result<T> {
    Err(IbkrError::InvalidOrderRequest(message.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_vwap_order_payload() {
        let request = example_input().into_place_orders_request().unwrap();
        let order_json = serde_json::to_value(&request.orders[0]).unwrap();

        assert_eq!(order_json["conid"], 72_539_702);
        assert_eq!(order_json["side"], "BUY");
        assert_eq!(order_json["orderType"], "LMT");
        assert_eq!(order_json["tif"], "DAY");
        assert_eq!(order_json["strategy"], "Vwap");
        assert_eq!(order_json["strategyParameters"]["maxPctVol"], "0.1");
        assert_eq!(
            order_json["strategyParameters"]["startTime"],
            "15:30:00 US/Eastern"
        );
    }

    #[test]
    fn rejects_non_positive_limit_price() {
        let mut input = example_input();
        input.limit_price = 0.0;

        assert!(input.into_place_orders_request().is_err());
    }

    #[test]
    fn rejects_non_numeric_max_percent_volume() {
        let mut input = example_input();
        input.max_percent_volume = "fast".to_string();

        assert!(input.into_place_orders_request().is_err());
    }

    fn example_input() -> VwapOrderInput {
        VwapOrderInput {
            conid: 72_539_702,
            side: OrderSide::Buy,
            quantity: 10.0,
            limit_price: 70.0,
            account_id: "KAPPA".to_string(),
            start_time: "15:30:00 US/Eastern".to_string(),
            end_time: "16:00:00 US/Eastern".to_string(),
            max_percent_volume: "0.1".to_string(),
            client_order_id: "kappa-k1-TQQQ-20260723-buy-new-10-7000".to_string(),
        }
    }
}
