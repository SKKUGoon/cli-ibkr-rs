use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{IbkrError, Result};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conid: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Value>,
    #[serde(
        rename = "orderType",
        alias = "order_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub order_type: Option<String>,
    #[serde(
        rename = "acctId",
        alias = "acct_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub acct_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conidex: Option<String>,
    #[serde(
        rename = "manualIndicator",
        alias = "manual_indicator",
        skip_serializing_if = "Option::is_none"
    )]
    pub manual_indicator: Option<bool>,
    #[serde(
        rename = "extOperator",
        alias = "ext_operator",
        skip_serializing_if = "Option::is_none"
    )]
    pub ext_operator: Option<String>,
    #[serde(
        rename = "secType",
        alias = "sec_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub sec_type: Option<String>,
    #[serde(
        rename = "cOID",
        alias = "coid",
        skip_serializing_if = "Option::is_none"
    )]
    pub coid: Option<String>,
    #[serde(
        rename = "parentId",
        alias = "parent_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_id: Option<String>,
    #[serde(
        rename = "listingExchange",
        alias = "listing_exchange",
        skip_serializing_if = "Option::is_none"
    )]
    pub listing_exchange: Option<String>,
    #[serde(
        rename = "isSingleGroup",
        alias = "is_single_group",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_single_group: Option<bool>,
    #[serde(
        rename = "outsideRTH",
        alias = "outside_rth",
        skip_serializing_if = "Option::is_none"
    )]
    pub outside_rth: Option<bool>,
    #[serde(
        rename = "auxPrice",
        alias = "aux_price",
        skip_serializing_if = "Option::is_none"
    )]
    pub aux_price: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tif: Option<String>,
    #[serde(
        rename = "trailingAmt",
        alias = "trailing_amt",
        skip_serializing_if = "Option::is_none"
    )]
    pub trailing_amt: Option<Value>,
    #[serde(
        rename = "trailingType",
        alias = "trailing_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub trailing_type: Option<String>,
    #[serde(
        rename = "customerAccount",
        alias = "customer_account",
        skip_serializing_if = "Option::is_none"
    )]
    pub customer_account: Option<String>,
    #[serde(
        rename = "isProCustomer",
        alias = "is_pro_customer",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_pro_customer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(
        rename = "cashQty",
        alias = "cash_qty",
        skip_serializing_if = "Option::is_none"
    )]
    pub cash_qty: Option<Value>,
    #[serde(
        rename = "fxQty",
        alias = "fx_qty",
        skip_serializing_if = "Option::is_none"
    )]
    pub fx_qty: Option<Value>,
    #[serde(
        rename = "useAdaptive",
        alias = "use_adaptive",
        skip_serializing_if = "Option::is_none"
    )]
    pub use_adaptive: Option<bool>,
    #[serde(
        rename = "isCcyConv",
        alias = "is_ccy_conv",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_ccy_conv: Option<bool>,
    #[serde(
        rename = "allocationMethod",
        alias = "allocation_method",
        skip_serializing_if = "Option::is_none"
    )]
    pub allocation_method: Option<String>,
    #[serde(
        rename = "manualOrderTime",
        alias = "manual_order_time",
        skip_serializing_if = "Option::is_none"
    )]
    pub manual_order_time: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(
        rename = "strategyParameters",
        alias = "strategy_parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub strategy_parameters: Option<Value>,
    #[serde(
        rename = "isClose",
        alias = "is_close",
        skip_serializing_if = "Option::is_none"
    )]
    pub is_close: Option<bool>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlaceOrdersRequest {
    pub orders: Vec<OrderRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplyRequest {
    pub confirmed: bool,
}

pub type Answers = BTreeMap<String, bool>;

impl OrderRequest {
    pub fn validate(&self) -> Result<()> {
        if self.conid.is_some() && self.conidex.is_some() {
            return Err(IbkrError::InvalidOrderRequest(
                "conid and conidex cannot both be provided".to_string(),
            ));
        }
        Ok(())
    }
}

impl PlaceOrdersRequest {
    pub fn new(orders: Vec<OrderRequest>) -> Result<Self> {
        if orders.is_empty() {
            return Err(IbkrError::InvalidOrderRequest(
                "orders file must contain at least one order".to_string(),
            ));
        }
        for order in &orders {
            order.validate()?;
        }
        Ok(Self { orders })
    }
}

pub fn parse_orders_json(input: &str) -> Result<PlaceOrdersRequest> {
    let value: Value = serde_json::from_str(input)?;
    let orders = if value.is_array() {
        serde_json::from_value(value)?
    } else {
        vec![serde_json::from_value(value)?]
    };
    PlaceOrdersRequest::new(orders)
}

pub fn parse_order_json(input: &str) -> Result<OrderRequest> {
    let order: OrderRequest = serde_json::from_str(input)?;
    order.validate()?;
    Ok(order)
}

pub fn parse_answers_json(input: &str) -> Result<Answers> {
    Ok(serde_json::from_str(input)?)
}

pub fn find_answer<'a>(
    message: &str,
    message_id: Option<&str>,
    answers: &'a Answers,
) -> Option<&'a bool> {
    if let Some(message_id) = message_id {
        if let Some(answer) = answers.get(message_id) {
            return Some(answer);
        }
    }
    answers
        .iter()
        .find_map(|(key, answer)| message.contains(key).then_some(answer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_ibkr_field_names() {
        let request = parse_orders_json(
            r#"{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","acct_id":"DU123","coid":"x","outside_rth":true,"strategy_parameters":{"maxPctVol":"0.1"}}"#,
        )
        .unwrap();

        let value = serde_json::to_value(request).unwrap();
        let order = &value["orders"][0];
        assert_eq!(order["orderType"], "LMT");
        assert_eq!(order["acctId"], "DU123");
        assert_eq!(order["cOID"], "x");
        assert_eq!(order["outsideRTH"], true);
        assert_eq!(order["strategyParameters"]["maxPctVol"], "0.1");
    }

    #[test]
    fn parses_single_object_and_array() {
        let single = parse_orders_json(r#"{"conid":1,"side":"BUY"}"#).unwrap();
        assert_eq!(single.orders.len(), 1);

        let multiple = parse_orders_json(r#"[{"conid":1},{"conid":2}]"#).unwrap();
        assert_eq!(multiple.orders.len(), 2);
    }

    #[test]
    fn rejects_conid_with_conidex() {
        let err = parse_orders_json(r#"{"conid":1,"conidex":"1@SMART"}"#).unwrap_err();
        assert!(matches!(err, IbkrError::InvalidOrderRequest(_)));
    }

    #[test]
    fn matches_message_id_before_substring() {
        let answers = parse_answers_json(r#"{"o354":true,"missing data":false}"#).unwrap();
        assert_eq!(
            find_answer("missing data", Some("o354"), &answers),
            Some(&true)
        );
    }

    #[test]
    fn matches_message_substring() {
        let answers = parse_answers_json(r#"{"Percentage constraint":true}"#).unwrap();
        assert_eq!(
            find_answer(
                "price exceeds the Percentage constraint of 3%",
                None,
                &answers
            ),
            Some(&true)
        );
    }
}
