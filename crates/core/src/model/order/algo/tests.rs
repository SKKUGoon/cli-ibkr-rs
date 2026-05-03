use serde_json::json;

use super::*;
use crate::IbkrError;

#[test]
fn builds_algo_query_params() {
    let request = AlgoParamsRequest::new(
        "265598",
        vec!["Adaptive".to_string(), "Vwap".to_string()],
        true,
        true,
    )
    .unwrap();

    assert_eq!(request.conid(), "265598");
    assert_eq!(
        request.params(),
        vec![
            ("algos".to_string(), "Adaptive;Vwap".to_string()),
            ("addDescription".to_string(), "1".to_string()),
            ("addParams".to_string(), "1".to_string()),
        ]
    );
}

#[test]
fn omits_empty_algo_filter() {
    let request = AlgoParamsRequest::new("265598", Vec::new(), false, false).unwrap();

    assert_eq!(
        request.params(),
        vec![
            ("addDescription".to_string(), "0".to_string()),
            ("addParams".to_string(), "0".to_string()),
        ]
    );
}

#[test]
fn rejects_empty_conid() {
    let err = AlgoParamsRequest::new(" ", Vec::new(), false, false).unwrap_err();

    assert!(matches!(err, IbkrError::InvalidOrderRequest(_)));
    assert!(err.to_string().contains("conid cannot be empty"));
}

#[test]
fn rejects_more_than_eight_algos() {
    let err = AlgoParamsRequest::new(
        "265598",
        (0..9).map(|index| format!("Algo{index}")).collect(),
        false,
        false,
    )
    .unwrap_err();

    assert!(matches!(err, IbkrError::InvalidOrderRequest(_)));
    assert!(err.to_string().contains("more than 8"));
}

#[test]
fn deserializes_documented_algo_response() {
    let response: AlgoParamsResponse = serde_json::from_value(json!({
        "algos": [
            {
                "name": "Adaptive",
                "id": "Adaptive",
                "parameters": [{
                    "guiRank": 1,
                    "defaultValue": "Normal",
                    "name": "Adaptive order priority/urgency",
                    "id": "adaptivePriority",
                    "legalStrings": ["Urgent", "Normal", "Patient"],
                    "required": "true",
                    "valueClassName": "String",
                    "customField": "kept"
                }]
            },
            {
                "name": "VWAP",
                "id": "Vwap"
            }
        ]
    }))
    .unwrap();

    assert_eq!(response.algos[0].parameters[0].gui_rank, Some(1));
    assert_eq!(response.algos[0].parameters[0].legal_strings.len(), 3);
    assert_eq!(response.algos[0].parameters[0].extra["customField"], "kept");
    assert!(response.algos[1].parameters.is_empty());
}
