use ibkr_core::model::history::HistoryRequest;
use ibkr_core::model::stock::StockConidRequest;
use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::cli::args::{FetchHistoryArgs, StockConidArgs};
use crate::error::WorkerError;

pub async fn fetch_history(
    client: &IbkrClient,
    args: FetchHistoryArgs,
) -> Result<Value, WorkerError> {
    let request = HistoryRequest {
        conid: args.conid,
        period: args.period,
        bar: args.bar,
        exchange: args.exchange,
        outside_rth: args.outside_rth,
        start_time: args.start_time,
    };
    Ok(client.fetch_history(&request).await?)
}

pub async fn stock_conid(client: &IbkrClient, args: StockConidArgs) -> Result<Value, WorkerError> {
    let request = StockConidRequest {
        symbol: args.symbol,
        exchange: args.exchange,
        default_filtering: args.default_filtering,
    };
    Ok(client.stock_conid(&request).await?)
}
