use std::env;
use std::fs;
use std::path::Path;

use ibkr_core::model::order::{
    default_answers, merge_answers, parse_answers_json, parse_order_json, parse_orders_json,
    Answers, OrderRequest, PlaceOrdersRequest,
};

use crate::error::WorkerError;

const ANSWERS_BASE_FILE_ENV: &str = "IBKR_ORDERS_ANSWER_JSON";

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
    let env_base_path = env::var_os(ANSWERS_BASE_FILE_ENV);
    read_answers_input_with_base_path(path, json, env_base_path.as_deref().map(Path::new))
}

fn read_optional_answers_file(path: &Path) -> Result<Answers, WorkerError> {
    if path.as_os_str().is_empty() || !path.try_exists()? {
        return Ok(Answers::new());
    }

    read_answers_file(path)
}

fn read_answers_input_with_base_path(
    path: Option<&Path>,
    json: Option<&str>,
    base_path: Option<&Path>,
) -> Result<Answers, WorkerError> {
    let mut answers = default_answers();

    if let Some(base_path) = base_path {
        answers = merge_answers(answers, read_optional_answers_file(base_path)?);
    }

    if let Some(path) = path {
        answers = merge_answers(answers, read_answers_file(path)?);
    }

    if let Some(json) = json {
        answers = merge_answers(answers, parse_answers_json(json)?);
    }

    Ok(answers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn unique_test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "ibkrctl-{name}-{}-{}.json",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ))
    }

    #[test]
    fn answers_input_uses_defaults_when_omitted() {
        let answers = read_answers_input_with_base_path(None, None, None).unwrap();

        assert_eq!(answers.get("o163"), Some(&true));
        assert_eq!(answers.get("o451"), Some(&true));
        assert_eq!(answers.get("o354"), Some(&true));
        assert_eq!(answers.get("o10331"), Some(&true));
    }

    #[test]
    fn answers_input_overrides_defaults_and_extends_map() {
        let answers = read_answers_input_with_base_path(
            None,
            Some(r#"{"o354":false,"custom warning":true}"#),
            None,
        )
        .unwrap();

        assert_eq!(answers.get("o354"), Some(&false));
        assert_eq!(answers.get("custom warning"), Some(&true));
        assert_eq!(answers.get("o163"), Some(&true));
    }

    #[test]
    fn missing_base_answers_file_falls_back_to_empty() {
        let missing = unique_test_path("missing-base");
        let answers = read_answers_input_with_base_path(None, None, Some(&missing)).unwrap();

        assert_eq!(answers, default_answers());
    }

    #[test]
    fn existing_base_answers_file_merges_into_defaults() {
        let path = unique_test_path("base");
        fs::write(&path, r#"{"base warning":true,"o354":false}"#).unwrap();

        let answers = read_answers_input_with_base_path(None, None, Some(&path)).unwrap();

        assert_eq!(answers.get("base warning"), Some(&true));
        assert_eq!(answers.get("o354"), Some(&false));
        assert_eq!(answers.get("o163"), Some(&true));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn inline_answers_override_base_answers_file() {
        let path = unique_test_path("override");
        fs::write(&path, r#"{"o354":false,"base warning":true}"#).unwrap();

        let answers =
            read_answers_input_with_base_path(None, Some(r#"{"o354":true}"#), Some(&path)).unwrap();

        assert_eq!(answers.get("o354"), Some(&true));
        assert_eq!(answers.get("base warning"), Some(&true));

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn invalid_existing_base_answers_file_fails() {
        let path = unique_test_path("invalid");
        fs::write(&path, "{not json").unwrap();

        let err = read_answers_input_with_base_path(None, None, Some(&path)).unwrap_err();

        assert!(matches!(
            err,
            WorkerError::Ibkr(ibkr_core::IbkrError::Json(_))
        ));

        fs::remove_file(path).unwrap();
    }
}
