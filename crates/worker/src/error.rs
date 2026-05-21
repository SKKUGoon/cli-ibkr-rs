use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("{0}")]
    Ibkr(#[from] ibkr_core::IbkrError),

    #[error("missing required config value: {0}")]
    MissingConfig(&'static str),

    #[error("invalid environment variable {key}={value:?}: {source}")]
    InvalidEnv {
        key: &'static str,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error("invalid cache mode {0:?}: expected redis, memory, or disabled")]
    InvalidCacheMode(String),

    #[error("refusing to overwrite existing OAuth material: {0}")]
    RefuseOverwrite(PathBuf),

    #[error("failed to create directory {path}: {source}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to set private permissions on {path}: {source}")]
    SetPermissions {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("OpenSSL error: {0}")]
    OpenSsl(String),

    #[error("failed to write output {path}: {source}")]
    WriteOutput {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid order input: {0}")]
    InvalidOrderInput(&'static str),

    #[error("order confirmation answer missing for message: {0}")]
    MissingOrderAnswer(String),

    #[error("order confirmation answer rejected message: {0}")]
    RejectedOrderAnswer(String),

    #[error("order confirmation response did not include a usable reply id: {0}")]
    MissingOrderReplyId(String),

    #[error(
        "order confirmation exceeded max replies ({max_replies}); last response: {last_response}"
    )]
    TooManyOrderReplies {
        max_replies: u32,
        last_response: String,
    },

    #[error("unexpected order response while handling confirmations: {0}")]
    UnexpectedOrderResponse(String),

    #[error("conid lookup for {symbol} matched {count} active database rows; add --exchange")]
    AmbiguousConid { symbol: String, count: usize },

    #[error("history bars parse failed: {0}")]
    HistoryParse(String),

    #[error("Postgres request failed: {0}")]
    Postgres(#[from] sqlx::Error),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),
}
