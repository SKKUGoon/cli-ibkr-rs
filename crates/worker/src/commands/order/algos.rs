use ibkr_core::model::order::AlgoParamsRequest;
use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::cli::OrderAlgosArgs;
use crate::error::WorkerError;

pub async fn fetch_order_algorithms(
    client: &IbkrClient,
    arguments: OrderAlgosArgs,
) -> Result<Value, WorkerError> {
    let request = AlgoParamsRequest::new(
        arguments.conid,
        arguments.algos,
        arguments.add_description,
        arguments.add_params,
    )?;

    Ok(client
        .order_algos(
            request.conid(),
            request.algos(),
            request.add_description(),
            request.add_params(),
        )
        .await?)
}
