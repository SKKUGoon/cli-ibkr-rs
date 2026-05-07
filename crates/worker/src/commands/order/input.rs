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

pub fn read_orders_input(
    path: Option<&Path>,
    json: Option<&str>,
) -> Result<PlaceOrdersRequest, WorkerError> {
    match (path, json) {
        (Some(path), None) => read_orders_file(path),
        (None, Some(json)) => Ok(parse_orders_json(json)?),
        _ => Err(WorkerError::InvalidOrderInput(
            "provide exactly one of --orders-file or --orders-json",
        )),
    }
}

pub fn read_order_file(path: &Path) -> Result<OrderRequest, WorkerError> {
    Ok(parse_order_json(&fs::read_to_string(path)?)?)
}

pub fn read_order_input(
    path: Option<&Path>,
    json: Option<&str>,
) -> Result<OrderRequest, WorkerError> {
    match (path, json) {
        (Some(path), None) => read_order_file(path),
        (None, Some(json)) => Ok(parse_order_json(json)?),
        _ => Err(WorkerError::InvalidOrderInput(
            "provide exactly one of --order-file or --order-json",
        )),
    }
}

pub fn read_answers_file(path: &Path) -> Result<Answers, WorkerError> {
    Ok(parse_answers_json(&fs::read_to_string(path)?)?)
}

pub fn read_answers_input(path: Option<&Path>, json: Option<&str>) -> Result<Answers, WorkerError> {
    match (path, json) {
        (Some(path), None) => read_answers_file(path),
        (None, Some(json)) => Ok(parse_answers_json(json)?),
        _ => Err(WorkerError::InvalidOrderInput(
            "provide exactly one of --answers-file or --answers-json",
        )),
    }
}
