use ibkr_core::model::stock::StockLookupResult;
use sqlx::{FromRow, PgPool};

use crate::error::WorkerError;

#[derive(Debug, Clone, FromRow, PartialEq, Eq)]
pub struct ConidRow {
    pub symbol: String,
    pub name_: Option<String>,
    pub conid: i64,
    pub exchange: Option<String>,
}

impl ConidRow {
    pub fn into_lookup_result(self) -> StockLookupResult {
        StockLookupResult {
            symbol: self.symbol,
            name_: self.name_,
            conid: self.conid,
            exchange: self.exchange,
        }
    }
}

pub async fn find_active_conid(
    pool: &PgPool,
    symbol: &str,
    exchange: Option<&str>,
) -> Result<Option<ConidRow>, WorkerError> {
    let rows = sqlx::query_as::<_, ConidRow>(
        r#"
        SELECT symbol, name_, conid, exchange
        FROM warehouse.conids
        WHERE upper(symbol) = $1
          AND active = true
          AND ($2::text IS NULL OR exchange IS NOT NULL AND lower(exchange) = lower($2))
        ORDER BY symbol, conid
        "#,
    )
    .bind(symbol.to_ascii_uppercase())
    .bind(exchange)
    .fetch_all(pool)
    .await?;

    select_one(rows, symbol)
}

pub async fn upsert_conid(
    pool: &PgPool,
    result: &StockLookupResult,
) -> Result<ConidRow, WorkerError> {
    let row = sqlx::query_as::<_, ConidRow>(
        r#"
        INSERT INTO warehouse.conids (symbol, name_, conid, exchange, active)
        VALUES ($1, $2, $3, $4, true)
        ON CONFLICT (conid) DO UPDATE
        SET symbol = EXCLUDED.symbol,
            name_ = EXCLUDED.name_,
            exchange = EXCLUDED.exchange,
            active = true,
            updated_at = now()
        RETURNING symbol, name_, conid, exchange
        "#,
    )
    .bind(&result.symbol)
    .bind(&result.name_)
    .bind(result.conid)
    .bind(&result.exchange)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

fn select_one(rows: Vec<ConidRow>, symbol: &str) -> Result<Option<ConidRow>, WorkerError> {
    match rows.len() {
        0 => Ok(None),
        1 => Ok(rows.into_iter().next()),
        count => Err(WorkerError::AmbiguousConid {
            symbol: symbol.to_ascii_uppercase(),
            count,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_one_returns_none_for_no_rows() {
        assert_eq!(select_one(Vec::new(), "AAPL").unwrap(), None);
    }

    #[test]
    fn select_one_returns_single_row() {
        let row = row("AAPL", 265598, "NASDAQ");

        assert_eq!(select_one(vec![row.clone()], "AAPL").unwrap(), Some(row));
    }

    #[test]
    fn select_one_rejects_multiple_rows() {
        let err = select_one(
            vec![row("AAPL", 265598, "NASDAQ"), row("AAPL", 38708077, "MEXI")],
            "aapl",
        )
        .unwrap_err();

        assert!(err.to_string().contains("matched 2 active database rows"));
        assert!(err.to_string().contains("AAPL"));
    }

    fn row(symbol: &str, conid: i64, exchange: &str) -> ConidRow {
        ConidRow {
            symbol: symbol.to_string(),
            name_: Some("Example".to_string()),
            conid,
            exchange: Some(exchange.to_string()),
        }
    }
}
