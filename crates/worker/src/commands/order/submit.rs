use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::cli::{
    OrderCancelArgs, OrderModifyArgs, OrderPlaceArgs, OrderReplyArgs, OrderStatusArgs,
    OrderWhatifArgs,
};
use crate::error::WorkerError;

use super::confirmations::handle_confirmations;
use super::input::{read_answers_file, read_order_file, read_orders_file};

pub async fn place(client: &IbkrClient, args: OrderPlaceArgs) -> Result<Value, WorkerError> {
    let request = read_orders_file(&args.orders_file)?;
    let answers = read_answers_file(&args.answers_file)?;
    let value = client.place_orders(&args.account_id, &request).await?;
    handle_confirmations(client, value, &answers, args.max_replies).await
}

pub async fn whatif(client: &IbkrClient, args: OrderWhatifArgs) -> Result<Value, WorkerError> {
    let request = read_orders_file(&args.orders_file)?;
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
    let request = read_order_file(&args.order_file)?;
    let answers = read_answers_file(&args.answers_file)?;
    let value = client
        .modify_order(&args.account_id, &args.order_id, &request)
        .await?;
    handle_confirmations(client, value, &answers, args.max_replies).await
}

pub async fn status(client: &IbkrClient, args: OrderStatusArgs) -> Result<Value, WorkerError> {
    Ok(client.order_status(&args.order_id).await?)
}
