use ibkr_core::model::order::AlgoParamsRequest;
use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::cli::OrderAlgosArgs;
use crate::error::WorkerError;

pub async fn run(client: &IbkrClient, args: OrderAlgosArgs) -> Result<Value, WorkerError> {
    let request = AlgoParamsRequest::new(
        args.conid,
        args.algos,
        args.add_description,
        args.add_params,
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
