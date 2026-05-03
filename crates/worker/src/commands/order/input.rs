use std::fs;
use std::path::Path;

use ibkr_core::model::order::{
    parse_answers_json, parse_order_json, parse_orders_json, Answers, OrderRequest,
    PlaceOrdersRequest,
};

use crate::error::WorkerError;

pub fn read_orders_file(path: &Path) -> Result<PlaceOrdersRequest, WorkerError> {
    Ok(parse_orders_json(&fs::read_to_string(path)?)?)
}

pub fn read_order_file(path: &Path) -> Result<OrderRequest, WorkerError> {
    Ok(parse_order_json(&fs::read_to_string(path)?)?)
}

pub fn read_answers_file(path: &Path) -> Result<Answers, WorkerError> {
    Ok(parse_answers_json(&fs::read_to_string(path)?)?)
}
