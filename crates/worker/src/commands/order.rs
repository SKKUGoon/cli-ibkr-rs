use std::fs;
use std::path::Path;

use ibkr_core::model::order::{
    find_answer, parse_answers_json, parse_order_json, parse_orders_json, Answers, OrderRequest,
    PlaceOrdersRequest,
};
use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::cli::{
    OrderAlgosArgs, OrderCancelArgs, OrderCommand, OrderModifyArgs, OrderPlaceArgs, OrderReplyArgs,
    OrderStatusArgs, OrderWhatifArgs,
};
use crate::error::WorkerError;

pub async fn run(client: &IbkrClient, command: OrderCommand) -> Result<Value, WorkerError> {
    match command {
        OrderCommand::Algos(args) => algos(client, args).await,
        OrderCommand::Place(args) => place(client, args).await,
        OrderCommand::Whatif(args) => whatif(client, args).await,
        OrderCommand::Reply(args) => reply(client, args).await,
        OrderCommand::Cancel(args) => cancel(client, args).await,
        OrderCommand::Modify(args) => modify(client, args).await,
        OrderCommand::Status(args) => status(client, args).await,
    }
}

async fn algos(client: &IbkrClient, args: OrderAlgosArgs) -> Result<Value, WorkerError> {
    Ok(client
        .order_algos(
            &args.conid,
            &args.algos,
            args.add_description,
            args.add_params,
        )
        .await?)
}

async fn place(client: &IbkrClient, args: OrderPlaceArgs) -> Result<Value, WorkerError> {
    let request = read_orders_file(&args.orders_file)?;
    let answers = read_answers_file(&args.answers_file)?;
    let value = client.place_orders(&args.account_id, &request).await?;
    handle_confirmations(client, value, &answers, args.max_replies).await
}

async fn whatif(client: &IbkrClient, args: OrderWhatifArgs) -> Result<Value, WorkerError> {
    let request = read_orders_file(&args.orders_file)?;
    Ok(client.whatif_order(&args.account_id, &request).await?)
}

async fn reply(client: &IbkrClient, args: OrderReplyArgs) -> Result<Value, WorkerError> {
    Ok(client.reply(&args.reply_id, args.confirmed).await?)
}

async fn cancel(client: &IbkrClient, args: OrderCancelArgs) -> Result<Value, WorkerError> {
    Ok(client
        .cancel_order(&args.account_id, &args.order_id)
        .await?)
}

async fn modify(client: &IbkrClient, args: OrderModifyArgs) -> Result<Value, WorkerError> {
    let request = read_order_file(&args.order_file)?;
    let answers = read_answers_file(&args.answers_file)?;
    let value = client
        .modify_order(&args.account_id, &args.order_id, &request)
        .await?;
    handle_confirmations(client, value, &answers, args.max_replies).await
}

async fn status(client: &IbkrClient, args: OrderStatusArgs) -> Result<Value, WorkerError> {
    Ok(client.order_status(&args.order_id).await?)
}

fn read_orders_file(path: &Path) -> Result<PlaceOrdersRequest, WorkerError> {
    Ok(parse_orders_json(&fs::read_to_string(path)?)?)
}

fn read_order_file(path: &Path) -> Result<OrderRequest, WorkerError> {
    Ok(parse_order_json(&fs::read_to_string(path)?)?)
}

fn read_answers_file(path: &Path) -> Result<Answers, WorkerError> {
    Ok(parse_answers_json(&fs::read_to_string(path)?)?)
}

async fn handle_confirmations(
    client: &IbkrClient,
    mut value: Value,
    answers: &Answers,
    max_replies: u32,
) -> Result<Value, WorkerError> {
    for _ in 0..max_replies {
        let Some(prompt) = order_prompt(&value)? else {
            return Ok(final_order_response(value));
        };

        match find_answer(&prompt.message, prompt.message_id.as_deref(), answers) {
            Some(true) => {
                value = client.reply(&prompt.reply_id, true).await?;
            }
            Some(false) => return Err(WorkerError::RejectedOrderAnswer(prompt.message)),
            None => return Err(WorkerError::MissingOrderAnswer(prompt.message)),
        }
    }

    Err(WorkerError::TooManyOrderReplies {
        max_replies,
        last_response: value.to_string(),
    })
}

fn final_order_response(value: Value) -> Value {
    match value {
        Value::Array(mut values) if values.len() == 1 => values.remove(0),
        other => other,
    }
}

#[derive(Debug, PartialEq, Eq)]
struct OrderPrompt {
    reply_id: String,
    message: String,
    message_id: Option<String>,
}

fn order_prompt(value: &Value) -> Result<Option<OrderPrompt>, WorkerError> {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn returns_none_when_response_is_final() {
        assert_eq!(order_prompt(&json!([{"success": true}])).unwrap(), None);
    }

    #[test]
    fn extracts_prompt() {
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
}
