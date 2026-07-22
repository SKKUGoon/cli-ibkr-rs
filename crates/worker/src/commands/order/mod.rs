mod algos;
pub(crate) mod confirmations;
mod input;
mod submit;

use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::cli::{OrderCommand, OrderFeePlanArgs};
use crate::error::WorkerError;

pub async fn execute_order_command(
    client: &IbkrClient,
    command: OrderCommand,
) -> Result<Value, WorkerError> {
    match command {
        OrderCommand::Algos(arguments) => algos::fetch_order_algorithms(client, arguments).await,
        OrderCommand::Place(args) => submit::place(client, args).await,
        OrderCommand::Whatif(args) => submit::whatif(client, args).await,
        OrderCommand::Reply(args) => submit::reply(client, args).await,
        OrderCommand::Cancel(args) => submit::cancel(client, args).await,
        OrderCommand::Modify(args) => submit::modify(client, args).await,
        OrderCommand::Status(args) => submit::status(client, args).await,
        OrderCommand::FeePlan(args) => submit::fee_plan(args).await,
    }
}

pub async fn calculate_order_fee_plan(arguments: OrderFeePlanArgs) -> Result<Value, WorkerError> {
    submit::fee_plan(arguments).await
}
