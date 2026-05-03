use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum IbkrError {
    #[error("missing required config value: {0}")]
    MissingConfig(&'static str),

    #[error("invalid private key format in {path}: expected {expected}")]
    InvalidKeyFormat {
        path: PathBuf,
        expected: &'static str,
    },

    #[error("failed to read {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse RSA key {path}: {message}")]
    RsaKey { path: PathBuf, message: String },

    #[error("failed to parse DH params {path}: {message}")]
    DhParams { path: PathBuf, message: String },

    #[error("crypto operation failed: {0}")]
    Crypto(String),

    #[error("IBKR returned HTTP {status}: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("stock conid lookup failed for {symbol}: {message}")]
    StockConidLookup { symbol: String, message: String },

    #[error("invalid order request: {0}")]
    InvalidOrderRequest(String),

    #[error("live session token cache error: {0}")]
    LiveSessionCache(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Redis request failed: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, IbkrError>;
