use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

// Mirror of the IBKR `iserver/account/{accountId}/orders` order body.
//
// Wire format is camelCase by default (`rename_all = "camelCase"`). Only fields
// whose IBKR spelling differs from a plain camelCase conversion carry an
// explicit `rename` (e.g. `cOID`, `outsideRTH`). Every renamed Rust field also
// accepts its `snake_case` form via `alias` so users can hand-write JSON in
// either convention.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    // --- Instrument identification ---------------------------------------
    // `conid` and `conidex` are mutually exclusive; `validate()` enforces it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conid: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conidex: Option<String>,
    #[serde(alias = "sec_type", skip_serializing_if = "Option::is_none")]
    pub sec_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticker: Option<String>,
    #[serde(alias = "listing_exchange", skip_serializing_if = "Option::is_none")]
    pub listing_exchange: Option<String>,

    // --- Core order parameters -------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(alias = "order_type", skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(alias = "acct_id", skip_serializing_if = "Option::is_none")]
    pub acct_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tif: Option<String>,

    // --- Sizing (mutually exclusive — see `validate()`) ------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Value>,
    #[serde(alias = "cash_qty", skip_serializing_if = "Option::is_none")]
    pub cash_qty: Option<Value>,
    #[serde(alias = "fx_qty", skip_serializing_if = "Option::is_none")]
    pub fx_qty: Option<Value>,

    // --- Pricing ---------------------------------------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Value>,
    #[serde(alias = "aux_price", skip_serializing_if = "Option::is_none")]
    pub aux_price: Option<Value>,
    #[serde(alias = "trailing_amt", skip_serializing_if = "Option::is_none")]
    pub trailing_amt: Option<Value>,
    #[serde(alias = "trailing_type", skip_serializing_if = "Option::is_none")]
    pub trailing_type: Option<String>,

    // --- Order grouping / linkage ----------------------------------------
    // `cOID` and `outsideRTH` use IBKR's non-standard casing so they need
    // explicit `rename`s; the camelCase default would emit `cOid`/`outsideRth`.
    #[serde(
        rename = "cOID",
        alias = "coid",
        skip_serializing_if = "Option::is_none"
    )]
    pub coid: Option<String>,
    #[serde(alias = "parent_id", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(alias = "is_single_group", skip_serializing_if = "Option::is_none")]
    pub is_single_group: Option<bool>,

    // --- Routing / session flags -----------------------------------------
    #[serde(
        rename = "outsideRTH",
        alias = "outside_rth",
        skip_serializing_if = "Option::is_none"
    )]
    pub outside_rth: Option<bool>,
    #[serde(alias = "use_adaptive", skip_serializing_if = "Option::is_none")]
    pub use_adaptive: Option<bool>,
    #[serde(alias = "is_ccy_conv", skip_serializing_if = "Option::is_none")]
    pub is_ccy_conv: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deactivated: Option<bool>,
    #[serde(alias = "is_close", skip_serializing_if = "Option::is_none")]
    pub is_close: Option<bool>,

    // --- Advisor / audit metadata ----------------------------------------
    #[serde(alias = "manual_indicator", skip_serializing_if = "Option::is_none")]
    pub manual_indicator: Option<bool>,
    #[serde(alias = "ext_operator", skip_serializing_if = "Option::is_none")]
    pub ext_operator: Option<String>,
    #[serde(alias = "customer_account", skip_serializing_if = "Option::is_none")]
    pub customer_account: Option<String>,
    #[serde(alias = "is_pro_customer", skip_serializing_if = "Option::is_none")]
    pub is_pro_customer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(alias = "allocation_method", skip_serializing_if = "Option::is_none")]
    pub allocation_method: Option<String>,
    #[serde(alias = "manual_order_time", skip_serializing_if = "Option::is_none")]
    pub manual_order_time: Option<Value>,

    // --- Algo strategy (strategyParameters requires strategy — see `validate()`) ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(alias = "strategy_parameters", skip_serializing_if = "Option::is_none")]
    pub strategy_parameters: Option<Value>,

    // Catch-all for forward-compatibility: any field IBKR adds that we don't
    // model yet round-trips through `extra` instead of being silently dropped.
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
