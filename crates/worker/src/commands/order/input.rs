use std::fs;
use std::path::Path;

use ibkr_core::model::order::{
    default_answers, merge_answers, parse_answers_json, parse_order_json, parse_orders_json,
    Answers, OrderRequest, PlaceOrdersRequest,
};

use crate::error::WorkerError;

pub fn read_orders_input(json: &str) -> Result<PlaceOrdersRequest, WorkerError> {
    Ok(parse_orders_json(json)?)
}

pub fn read_order_input(json: &str) -> Result<OrderRequest, WorkerError> {
    Ok(parse_order_json(json)?)
}

pub fn read_answers_file(path: &Path) -> Result<Answers, WorkerError> {
    Ok(parse_answers_json(&fs::read_to_string(path)?)?)
}

pub fn read_answers_input(path: Option<&Path>, json: Option<&str>) -> Result<Answers, WorkerError> {
    let overrides = match (path, json) {
        (Some(path), None) => Some(read_answers_file(path)?),
        (None, Some(json)) => Some(parse_answers_json(json)?),
        (None, None) => None,
        (Some(_), Some(_)) => {
            return Err(WorkerError::InvalidOrderInput(
                "provide at most one of --answers-file or --answers-json",
            ));
        }
    };

    Ok(match overrides {
        Some(overrides) => merge_answers(default_answers(), overrides),
        None => default_answers(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_input_uses_defaults_when_omitted() {
        let answers = read_answers_input(None, None).unwrap();

        assert_eq!(answers.get("o163"), Some(&true));
        assert_eq!(answers.get("o451"), Some(&true));
        assert_eq!(answers.get("o354"), Some(&true));
        assert_eq!(answers.get("o10331"), Some(&true));
    }

    #[test]
    fn answers_input_overrides_defaults_and_extends_map() {
        let answers =
            read_answers_input(None, Some(r#"{"o354":false,"custom warning":true}"#)).unwrap();

        assert_eq!(answers.get("o354"), Some(&false));
        assert_eq!(answers.get("custom warning"), Some(&true));
        assert_eq!(answers.get("o163"), Some(&true));
    }
}
