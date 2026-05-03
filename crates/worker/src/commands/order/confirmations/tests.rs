use serde_json::json;

use super::*;

#[test]
fn returns_none_when_response_is_final() {
    assert_eq!(order_prompt(&json!([{"success": true}])).unwrap(), None);
}

#[test]
fn extracts_prompt_with_numeric_reply_id_and_message_ids() {
    let prompt = order_prompt(&json!([{
        "id": 7,
        "message": [" price exceeds\nlimit "],
        "messageIds": ["o163"]
    }]))
    .unwrap()
    .unwrap();

    assert_eq!(
        prompt,
        OrderPrompt {
            reply_id: "7".to_string(),
            message: "price exceedslimit".to_string(),
            message_id: Some("o163".to_string()),
        }
    );
}

#[test]
fn extracts_prompt_with_string_reply_id_and_message_id() {
    let prompt = order_prompt(&json!([{
        "id": "abc",
        "message": " confirm ",
        "messageId": "o354"
    }]))
    .unwrap()
    .unwrap();

    assert_eq!(
        prompt,
        OrderPrompt {
            reply_id: "abc".to_string(),
            message: "confirm".to_string(),
            message_id: Some("o354".to_string()),
        }
    );
}

#[test]
fn rejects_prompt_without_reply_id() {
    let err = order_prompt(&json!([{"message": ["confirm"]}])).unwrap_err();
    assert!(matches!(err, WorkerError::MissingOrderReplyId(_)));
}

#[test]
fn collapses_single_final_array_value() {
    assert_eq!(
        final_order_response(json!([{"success": true}])),
        json!({"success": true})
    );
    assert_eq!(
        final_order_response(json!([{"a": 1}, {"b": 2}])),
        json!([{"a": 1}, {"b": 2}])
    );
}

#[test]
fn builds_max_replies_error() {
    let err = too_many_replies(0, json!([{"message": ["confirm"]}]));

    assert!(matches!(
        err,
        WorkerError::TooManyOrderReplies { max_replies: 0, .. }
    ));
    assert!(err.to_string().contains("exceeded max replies (0)"));
}
