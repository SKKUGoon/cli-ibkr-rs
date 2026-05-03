use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::error::WorkerError;

pub async fn accounts(client: &IbkrClient) -> Result<Value, WorkerError> {
    Ok(client.accounts().await?)
}

pub async fn account_summary(client: &IbkrClient, account_id: &str) -> Result<Value, WorkerError> {
    Ok(client.account_summary(account_id).await?)
}

pub async fn portfolio_summary(
    client: &IbkrClient,
    account_id: &str,
) -> Result<Value, WorkerError> {
    Ok(client.portfolio_summary(account_id).await?)
}

pub async fn ledger(client: &IbkrClient, account_id: &str) -> Result<Value, WorkerError> {
    Ok(client.ledger(account_id).await?)
}

pub async fn positions(
    client: &IbkrClient,
    account_id: &str,
    page: u32,
) -> Result<Value, WorkerError> {
    Ok(client.positions(account_id, page).await?)
}

pub async fn live_orders(
    client: &IbkrClient,
    account_id: Option<&str>,
    force: bool,
) -> Result<Value, WorkerError> {
    Ok(client.live_orders(account_id, force).await?)
}
