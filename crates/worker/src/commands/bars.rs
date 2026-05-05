use serde_json::Value;
use sqlx::PgPool;

use crate::error::WorkerError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryBar {
    pub conid: i64,
    pub timestamp_ms: i64,
    pub bar_length_seconds: i32,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: Option<i64>,
}

pub fn parse_history_bars(conid: &str, response: &Value) -> Result<Vec<HistoryBar>, WorkerError> {
    let conid = conid
        .parse::<i64>()
        .map_err(|source| WorkerError::HistoryBars {
            message: format!("invalid conid {conid:?}: {source}"),
        })?;

    let bar_length_seconds = value_i64(response, "barLength")
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| WorkerError::HistoryBars {
            message: "history response missing integer barLength".to_string(),
        })?;

    let data = response
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| WorkerError::HistoryBars {
            message: "history response missing data array".to_string(),
        })?;

    data.iter()
        .enumerate()
        .map(|(index, row)| parse_history_bar(conid, bar_length_seconds, index, row))
        .collect()
}

pub async fn upsert_history_bars(pool: &PgPool, bars: &[HistoryBar]) -> Result<u64, WorkerError> {
    let mut affected = 0;
    for bar in bars {
        let result = sqlx::query(
            r#"
            INSERT INTO warehouse.ibkr_bars (
                conid, bar_start, bar_end, open, high, low, close, volume
            )
            VALUES (
                $1,
                to_timestamp($2::double precision / 1000.0),
                to_timestamp($2::double precision / 1000.0) + make_interval(secs => $3),
                $4::numeric,
                $5::numeric,
                $6::numeric,
                $7::numeric,
                $8
            )
            ON CONFLICT (conid, bar_start, bar_end) DO UPDATE
            SET open = EXCLUDED.open,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                close = EXCLUDED.close,
                volume = EXCLUDED.volume
            "#,
        )
        .bind(bar.conid)
        .bind(bar.timestamp_ms)
        .bind(f64::from(bar.bar_length_seconds))
        .bind(&bar.open)
        .bind(&bar.high)
        .bind(&bar.low)
        .bind(&bar.close)
        .bind(bar.volume)
        .execute(pool)
        .await?;

        affected += result.rows_affected();
    }
    Ok(affected)
}

fn parse_history_bar(
    conid: i64,
    bar_length_seconds: i32,
    index: usize,
    row: &Value,
) -> Result<HistoryBar, WorkerError> {
    Ok(HistoryBar {
        conid,
        timestamp_ms: value_i64(row, "t").ok_or_else(|| field_error(index, "t"))?,
        bar_length_seconds,
        open: number_string(row, "o").ok_or_else(|| field_error(index, "o"))?,
        high: number_string(row, "h").ok_or_else(|| field_error(index, "h"))?,
        low: number_string(row, "l").ok_or_else(|| field_error(index, "l"))?,
        close: number_string(row, "c").ok_or_else(|| field_error(index, "c"))?,
        volume: value_i64(row, "v"),
    })
}

fn number_string(row: &Value, key: &str) -> Option<String> {
    row.get(key)
        .and_then(Value::as_number)
        .map(ToString::to_string)
}

fn value_i64(row: &Value, key: &str) -> Option<i64> {
    let value = row.get(key)?;
    if let Some(value) = value.as_i64() {
        return Some(value);
    }
    value.as_f64().map(|value| value.trunc() as i64)
}

fn field_error(index: usize, field: &str) -> WorkerError {
    WorkerError::HistoryBars {
        message: format!("history data[{index}] missing numeric {field}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_history_bars_maps_ibkr_response() {
        let response = json!({
            "barLength": 60,
            "data": [
                {"t": 1698796800000_i64, "o": 171.05, "h": 192.93, "l": 170.12, "c": 189.37, "v": 6942998.04},
                {"t": 1698796860000_i64, "o": 189.37, "h": 190.0, "l": 188.5, "c": 189.5}
            ]
        });

        let bars = parse_history_bars("265598", &response).unwrap();

        assert_eq!(
            bars,
            vec![
                HistoryBar {
                    conid: 265598,
                    timestamp_ms: 1698796800000,
                    bar_length_seconds: 60,
                    open: "171.05".to_string(),
                    high: "192.93".to_string(),
                    low: "170.12".to_string(),
                    close: "189.37".to_string(),
                    volume: Some(6942998),
                },
                HistoryBar {
                    conid: 265598,
                    timestamp_ms: 1698796860000,
                    bar_length_seconds: 60,
                    open: "189.37".to_string(),
                    high: "190.0".to_string(),
                    low: "188.5".to_string(),
                    close: "189.5".to_string(),
                    volume: None,
                },
            ]
        );
    }

    #[test]
    fn parse_history_bars_rejects_missing_data() {
        let err = parse_history_bars("265598", &json!({"barLength": 60})).unwrap_err();

        assert!(err.to_string().contains("missing data array"));
    }

    #[test]
    fn parse_history_bars_rejects_malformed_row() {
        let response = json!({"barLength": 60, "data": [{"t": 1698796800000_i64}]});

        let err = parse_history_bars("265598", &response).unwrap_err();

        assert!(err.to_string().contains("data[0] missing numeric o"));
    }
}
