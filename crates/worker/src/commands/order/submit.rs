use ibkr_core::model::order::FEE_PLAN_NOTIONAL_THRESHOLD;
use ibkr_core::IbkrClient;
use serde_json::{json, Value};

use crate::cli::{
    OrderCancelArgs, OrderFeePlanArgs, OrderModifyArgs, OrderPlaceArgs, OrderReplyArgs,
    OrderStatusArgs, OrderWhatifArgs,
};
use crate::error::WorkerError;

use super::confirmations::handle_confirmations;
use super::input::{read_answers_input, read_order_input, read_orders_input};

pub async fn place(client: &IbkrClient, args: OrderPlaceArgs) -> Result<Value, WorkerError> {
    let request = read_orders_input(args.orders_file.as_deref(), args.orders_json.as_deref())?;
    let answers = read_answers_input(args.answers_file.as_deref(), args.answers_json.as_deref())?;
    let value = client.place_orders(&args.account_id, &request).await?;
    handle_confirmations(client, value, &answers, args.max_replies).await
}

pub async fn whatif(client: &IbkrClient, args: OrderWhatifArgs) -> Result<Value, WorkerError> {
    let request = read_orders_input(args.orders_file.as_deref(), args.orders_json.as_deref())?;
    Ok(client.whatif_order(&args.account_id, &request).await?)
}

pub async fn reply(client: &IbkrClient, args: OrderReplyArgs) -> Result<Value, WorkerError> {
    Ok(client.reply(&args.reply_id, args.confirmed).await?)
}

pub async fn cancel(client: &IbkrClient, args: OrderCancelArgs) -> Result<Value, WorkerError> {
    Ok(client
        .cancel_order(&args.account_id, &args.order_id)
        .await?)
}

pub async fn modify(client: &IbkrClient, args: OrderModifyArgs) -> Result<Value, WorkerError> {
    let request = read_order_input(args.order_file.as_deref(), args.order_json.as_deref())?;
    let answers = read_answers_input(args.answers_file.as_deref(), args.answers_json.as_deref())?;
    let value = client
        .modify_order(&args.account_id, &args.order_id, &request)
        .await?;
    handle_confirmations(client, value, &answers, args.max_replies).await
}

pub async fn status(client: &IbkrClient, args: OrderStatusArgs) -> Result<Value, WorkerError> {
    Ok(client.order_status(&args.order_id).await?)
}

pub async fn fee_plan(args: OrderFeePlanArgs) -> Result<Value, WorkerError> {
    let request = read_orders_input(args.orders_file.as_deref(), args.orders_json.as_deref())?;
    let decisions = request
        .orders
        .iter()
        .enumerate()
        .map(|(index, order)| {
            let notional = order
                .trading_notional()
                .ok_or(WorkerError::InvalidOrderInput(
                    "fee-plan requires numeric quantity and limit price fields on every order",
                ))?;
            let fee_plan = order
                .automatic_fee_plan()
                .ok_or(WorkerError::InvalidOrderInput(
                    "fee-plan requires numeric quantity and limit price fields on every order",
                ))?;
            Ok(json!({
                "index": index,
                "notional": notional,
                "threshold": FEE_PLAN_NOTIONAL_THRESHOLD,
                "feePlan": fee_plan,
            }))
        })
        .collect::<Result<Vec<_>, WorkerError>>()?;

    Ok(json!({ "orders": decisions }))
}
