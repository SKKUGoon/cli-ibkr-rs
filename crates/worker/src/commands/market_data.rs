use ibkr_core::model::history::HistoryRequest;
use ibkr_core::model::stock::StockConidRequest;
use ibkr_core::IbkrClient;
use serde_json::Value;
use sqlx::PgPool;

use crate::cli::args::{FetchHistoryArgs, StockConidArgs};
use crate::error::WorkerError;

use super::{bars, conids};

pub async fn fetch_history(
    pool: Option<&PgPool>,
    client: &IbkrClient,
    args: FetchHistoryArgs,
) -> Result<Value, WorkerError> {
    let conid = args.conid;
    let request = HistoryRequest {
        conid: conid.clone(),
        period: args.period,
        bar: args.bar,
        exchange: args.exchange,
        outside_rth: args.outside_rth,
        start_time: args.start_time,
    };
    let value = client.fetch_history(&request).await?;

    if let Some(pool) = pool {
        match bars::parse_history_bars(&conid, &value) {
            Ok(parsed) => {
                if let Err(err) = bars::upsert_history_bars(pool, &parsed).await {
                    eprintln!(
                        "historical bars database upsert failed; returning IBKR API result: {err}"
                    );
                }
            }
            Err(err) => {
                eprintln!("historical bars parsing failed; returning IBKR API result: {err}");
            }
        }
    }

    Ok(value)
}

pub async fn stock_conid_cached(
    pool: Option<&PgPool>,
    client: &IbkrClient,
    args: StockConidArgs,
) -> Result<Value, WorkerError> {
    let request = StockConidRequest {
        symbol: args.symbol.to_ascii_uppercase(),
        exchange: args.exchange,
        default_filtering: args.default_filtering,
    };

    if let Some(pool) = pool {
        match conids::find_active_conid(pool, &request.symbol, request.exchange.as_deref()).await {
            Ok(Some(row)) => return Ok(serde_json::to_value(row.into_lookup_result())?),
            Ok(None) => {}
            Err(err @ WorkerError::ConidLookup { .. }) => return Err(err),
            Err(err) => {
                eprintln!("conid database lookup failed; falling back to IBKR API: {err}");
            }
        }
    }

    let result = client.stock_lookup(&request).await?;
    if let Some(pool) = pool {
        match conids::upsert_conid(pool, &result).await {
            Ok(row) => return Ok(serde_json::to_value(row.into_lookup_result())?),
            Err(err) => {
                eprintln!("conid database upsert failed; returning IBKR API result: {err}");
            }
        }
    }
    Ok(serde_json::to_value(result)?)
}
