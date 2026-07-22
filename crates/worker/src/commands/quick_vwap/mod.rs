mod client_order_id;
mod prompt;

use chrono::Local;
use dialoguer::Confirm;
use ibkr_core::model::order::VwapOrderInput;
use ibkr_core::model::stock::StockConidRequest;
use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::cli::QuickVwapOrderArgs;
use crate::error::WorkerError;

use super::order::confirmations::handle_interactive_confirmations;
use client_order_id::build_vwap_client_order_id;
use prompt::prompt_for_vwap_order;

pub async fn prompt_and_submit_vwap_order(
    client: &IbkrClient,
    arguments: QuickVwapOrderArgs,
) -> Result<Value, WorkerError> {
    let selections = prompt_for_vwap_order(arguments)?;
    let stock_lookup = client
        .stock_lookup(&StockConidRequest {
            symbol: selections.ticker.clone(),
            exchange: selections.exchange.clone(),
            default_filtering: true,
        })
        .await?;
    let client_order_id = build_vwap_client_order_id(
        &selections.client_order_id_prefix,
        &stock_lookup.symbol,
        Local::now().date_naive(),
        selections.side,
        selections.quantity,
        selections.limit_price,
    );
    let account_id = selections.account_id.clone();
    let max_replies = selections.max_replies;
    let order_request = VwapOrderInput {
        conid: stock_lookup.conid,
        side: selections.side,
        quantity: selections.quantity,
        limit_price: selections.limit_price,
        account_id: account_id.clone(),
        start_time: selections.start_time,
        end_time: selections.end_time,
        max_percent_volume: selections.max_percent_volume,
        client_order_id,
    }
    .into_place_orders_request()?;

    eprintln!(
        "Resolved {} to conid {} on {}.",
        stock_lookup.symbol,
        stock_lookup.conid,
        stock_lookup
            .exchange
            .as_deref()
            .unwrap_or("unspecified exchange")
    );
    eprintln!(
        "VWAP order payload:\n{}",
        serde_json::to_string_pretty(&order_request)?
    );
    let should_submit = Confirm::new()
        .with_prompt("Submit this VWAP order?")
        .default(false)
        .interact()?;
    if !should_submit {
        return Err(WorkerError::OperatorCancelled);
    }

    let submission_response = client.place_orders(&account_id, &order_request).await?;
    handle_interactive_confirmations(client, submission_response, max_replies).await
}
