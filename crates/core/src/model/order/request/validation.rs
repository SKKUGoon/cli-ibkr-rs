use crate::{IbkrError, Result};

use super::OrderRequest;

impl OrderRequest {
    pub fn validate(&self) -> Result<()> {
        reject_pair(
            self.conid.is_some(),
            self.conidex.is_some(),
            "conid",
            "conidex",
        )?;
        reject_pair(
            self.quantity.is_some(),
            self.cash_qty.is_some(),
            "quantity",
            "cashQty",
        )?;
        reject_pair(
            self.quantity.is_some(),
            self.fx_qty.is_some(),
            "quantity",
            "fxQty",
        )?;
        reject_pair(
            self.cash_qty.is_some(),
            self.fx_qty.is_some(),
            "cashQty",
            "fxQty",
        )?;
        if self.strategy_parameters.is_some() && self.strategy.is_none() {
            return invalid("strategyParameters cannot be provided without strategy");
        }
        Ok(())
    }
}

fn reject_pair(left_present: bool, right_present: bool, left: &str, right: &str) -> Result<()> {
    if left_present && right_present {
        return invalid(&format!("{left} and {right} cannot both be provided"));
    }
    Ok(())
}

fn invalid<T>(message: &str) -> Result<T> {
    Err(IbkrError::InvalidOrderRequest(message.to_string()))
}
