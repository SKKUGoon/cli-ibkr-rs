use serde::Serialize;
use serde_json::Value;

use super::OrderRequest;

// IBKR switches commission schedules at this notional value (USD-equivalent).
// Below the threshold the tiered plan is cheaper; above it the fixed plan is.
pub const FEE_PLAN_NOTIONAL_THRESHOLD: f64 = 10_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum OrderFeePlan {
    Tiered,
    Fixed,
}

impl OrderRequest {
    pub fn trading_notional(&self) -> Option<f64> {
        let quantity = numeric_value(&self.quantity)?;
        let price = numeric_value(&self.price)?;
        Some((quantity * price).abs())
    }

    pub fn automatic_fee_plan(&self) -> Option<OrderFeePlan> {
        self.trading_notional().map(|notional| {
            if notional <= FEE_PLAN_NOTIONAL_THRESHOLD {
                OrderFeePlan::Tiered
            } else {
                OrderFeePlan::Fixed
            }
        })
    }
}

// Quantity and price arrive as serde_json::Value because IBKR accepts both
// JSON numbers and numeric strings on the same field. Anything else (null,
// bool, object) is treated as absent.
fn numeric_value(value: &Option<Value>) -> Option<f64> {
    match value.as_ref()? {
        Value::Number(number) => number.as_f64(),
        Value::String(value) => value.parse().ok(),
        _ => None,
    }
}
