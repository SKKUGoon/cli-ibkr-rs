use serde_json::Value;

use crate::{IbkrError, Result};

use super::{OrderRequest, PlaceOrdersRequest};

impl PlaceOrdersRequest {
    pub fn new(orders: Vec<OrderRequest>) -> Result<Self> {
        if orders.is_empty() {
            return invalid("orders file must contain at least one order");
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

fn invalid<T>(message: &str) -> Result<T> {
    Err(IbkrError::InvalidOrderRequest(message.to_string()))
}
