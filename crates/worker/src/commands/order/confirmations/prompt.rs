use serde_json::Value;

use crate::error::WorkerError;

pub(crate) fn final_order_response(value: Value) -> Value {
    match value {
        Value::Array(mut values) if values.len() == 1 => values.remove(0),
        other => other,
    }
}

pub(crate) fn too_many_replies(max_replies: u32, last_response: Value) -> WorkerError {
    WorkerError::TooManyOrderReplies {
        max_replies,
        last_response: last_response.to_string(),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OrderPrompt {
    pub reply_id: String,
    pub message: String,
    pub message_id: Option<String>,
}

impl OrderPrompt {
    pub(crate) fn describe(&self) -> String {
        match &self.message_id {
            Some(message_id) => format!("{} (messageId: {message_id})", self.message),
            None => format!("{} (messageId: not provided by IBKR)", self.message),
        }
    }
}

pub(crate) fn order_prompt(value: &Value) -> Result<Option<OrderPrompt>, WorkerError> {
    let Value::Array(items) = value else {
        return Err(WorkerError::UnexpectedOrderResponse(value.to_string()));
    };
    let Some(first) = items.first() else {
        return Ok(None);
    };

    let Some(messages) = first.get("message") else {
        return Ok(None);
    };
    let message = first_message(messages)
        .ok_or_else(|| WorkerError::UnexpectedOrderResponse(value.to_string()))?;
    let reply_id = first
        .get("id")
        .and_then(id_string)
        .ok_or_else(|| WorkerError::MissingOrderReplyId(value.to_string()))?;

    Ok(Some(OrderPrompt {
        reply_id,
        message,
        message_id: first_message_id(first),
    }))
}

fn first_message(value: &Value) -> Option<String> {
    match value {
        Value::Array(messages) => messages.first()?.as_str().map(clean_message),
        Value::String(message) => Some(clean_message(message)),
        _ => None,
    }
}

fn clean_message(message: &str) -> String {
    message.trim().replace('\n', "")
}

fn id_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn first_message_id(value: &Value) -> Option<String> {
    value.get("messageId").and_then(id_string).or_else(|| {
        value
            .get("messageIds")?
            .as_array()?
            .first()
            .and_then(id_string)
    })
}
