use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::error::WorkerError;

pub async fn auth_status(client: &IbkrClient) -> Result<Value, WorkerError> {
    Ok(client.auth_status().await?)
}

pub async fn init_session(client: &IbkrClient, compete: bool) -> Result<Value, WorkerError> {
    Ok(client.init_session(compete).await?)
}

pub async fn tickle(client: &IbkrClient) -> Result<Value, WorkerError> {
    Ok(client.tickle().await?)
}
