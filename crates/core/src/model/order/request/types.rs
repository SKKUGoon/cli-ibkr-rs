use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
