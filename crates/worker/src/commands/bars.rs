use std::collections::HashSet;

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

// Column-oriented projection of a batch of `HistoryBar`s, shaped so each
// `Vec` becomes one `UNNEST` parameter in `upsert_history_bars`. Producing
// this is pure (no DB), which is why the upsert path can be tested without
// a live Postgres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BarColumns {
    pub conid: Vec<i64>,
    pub timestamp_ms: Vec<i64>,
    pub bar_length_seconds: Vec<i32>,
    pub open: Vec<String>,
    pub high: Vec<String>,
    pub low: Vec<String>,
    pub close: Vec<String>,
    pub volume: Vec<Option<i64>>,
}

// Builds the UNNEST payload, deduplicating by the table's natural key so the
// generated statement never tries to touch the same row twice. We keep the
// LAST occurrence of each key, matching the previous loop's semantics (each
// `ON CONFLICT DO UPDATE` overwrote any earlier insert).
pub(crate) fn to_columns(bars: &[HistoryBar]) -> BarColumns {
    let mut seen: HashSet<(i64, i64, i32)> = HashSet::with_capacity(bars.len());
    let mut deduped: Vec<&HistoryBar> = Vec::with_capacity(bars.len());
    for bar in bars.iter().rev() {
        let key = (bar.conid, bar.timestamp_ms, bar.bar_length_seconds);
        if seen.insert(key) {
            deduped.push(bar);
        }
    }
    deduped.reverse();

    let n = deduped.len();
    let mut columns = BarColumns {
        conid: Vec::with_capacity(n),
        timestamp_ms: Vec::with_capacity(n),
        bar_length_seconds: Vec::with_capacity(n),
        open: Vec::with_capacity(n),
        high: Vec::with_capacity(n),
        low: Vec::with_capacity(n),
        close: Vec::with_capacity(n),
        volume: Vec::with_capacity(n),
    };
    for bar in deduped {
        columns.conid.push(bar.conid);
        columns.timestamp_ms.push(bar.timestamp_ms);
        columns.bar_length_seconds.push(bar.bar_length_seconds);
        columns.open.push(bar.open.clone());
        columns.high.push(bar.high.clone());
        columns.low.push(bar.low.clone());
        columns.close.push(bar.close.clone());
        columns.volume.push(bar.volume);
    }
    columns
}

pub fn parse_history_bars(conid: &str, response: &Value) -> Result<Vec<HistoryBar>, WorkerError> {
    let conid = conid.parse::<i64>().map_err(|source| {
        WorkerError::HistoryParse(format!("invalid conid {conid:?}: {source}"))
    })?;

    let bar_length_seconds = value_i64(response, "barLength")
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| {
            WorkerError::HistoryParse("history response missing integer barLength".to_string())
        })?;

    let data = response
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            WorkerError::HistoryParse("history response missing data array".to_string())
        })?;

    data.iter()
        .enumerate()
        .map(|(index, row)| parse_history_bar(conid, bar_length_seconds, index, row))
        .collect()
}

// One INSERT per call regardless of batch size: each column is sent as a
// Postgres array and `UNNEST` zips them back into rows. Pulls the row
// shaping out into `to_columns` so the data-massaging is covered by unit
// tests; only the actual DB round-trip stays untested without a live
// Postgres.
const UPSERT_HISTORY_BARS_SQL: &str = r#"
INSERT INTO warehouse.ibkr_bars (
    conid, bar_start, bar_end, open, high, low, close, volume
)
SELECT
    u.conid,
    to_timestamp(u.ts_ms::double precision / 1000.0),
    to_timestamp(u.ts_ms::double precision / 1000.0) + make_interval(secs => u.bar_secs),
    u.open::numeric,
    u.high::numeric,
    u.low::numeric,
    u.close::numeric,
    u.volume
FROM UNNEST(
    $1::bigint[],
    $2::bigint[],
    $3::double precision[],
    $4::text[],
    $5::text[],
    $6::text[],
    $7::text[],
    $8::bigint[]
) AS u(conid, ts_ms, bar_secs, open, high, low, close, volume)
ON CONFLICT (conid, bar_start, bar_end) DO UPDATE
SET open = EXCLUDED.open,
    high = EXCLUDED.high,
    low = EXCLUDED.low,
    close = EXCLUDED.close,
    volume = EXCLUDED.volume
"#;

pub async fn upsert_history_bars(pool: &PgPool, bars: &[HistoryBar]) -> Result<u64, WorkerError> {
    if bars.is_empty() {
        return Ok(0);
    }
    let cols = to_columns(bars);
    // `bar_length_seconds` is sent as double precision because that matches
    // what `make_interval(secs => ...)` expects and avoids a server-side cast.
    let bar_secs: Vec<f64> = cols
        .bar_length_seconds
        .iter()
        .map(|seconds| f64::from(*seconds))
        .collect();

    let result = sqlx::query(UPSERT_HISTORY_BARS_SQL)
        .bind(&cols.conid)
        .bind(&cols.timestamp_ms)
        .bind(&bar_secs)
        .bind(&cols.open)
        .bind(&cols.high)
        .bind(&cols.low)
        .bind(&cols.close)
        .bind(&cols.volume)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
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
    WorkerError::HistoryParse(format!("history data[{index}] missing numeric {field}"))
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

    fn bar(conid: i64, ts_ms: i64, secs: i32, close: &str, volume: Option<i64>) -> HistoryBar {
        HistoryBar {
            conid,
            timestamp_ms: ts_ms,
            bar_length_seconds: secs,
            open: "0".to_string(),
            high: "0".to_string(),
            low: "0".to_string(),
            close: close.to_string(),
            volume,
        }
    }

    #[test]
    fn to_columns_preserves_input_order_when_unique() {
        let bars = vec![
            bar(1, 1_000, 60, "a", Some(10)),
            bar(1, 2_000, 60, "b", Some(20)),
            bar(2, 1_000, 60, "c", None),
        ];

        let cols = to_columns(&bars);

        assert_eq!(cols.conid.len(), 3);
        assert_eq!(cols.conid, vec![1, 1, 2]);
        assert_eq!(cols.timestamp_ms, vec![1_000, 2_000, 1_000]);
        assert_eq!(cols.bar_length_seconds, vec![60, 60, 60]);
        assert_eq!(cols.close, vec!["a", "b", "c"]);
        assert_eq!(cols.volume, vec![Some(10), Some(20), None]);
    }

    #[test]
    fn to_columns_deduplicates_keeping_last() {
        // Two rows share the natural key (conid=1, ts=1000, secs=60). The
        // second occurrence should survive; without dedup the UNNEST INSERT
        // would error with "ON CONFLICT DO UPDATE command cannot affect row
        // a second time".
        let bars = vec![
            bar(1, 1_000, 60, "first", Some(10)),
            bar(2, 1_000, 60, "other", Some(99)),
            bar(1, 1_000, 60, "last", Some(20)),
        ];

        let cols = to_columns(&bars);

        assert_eq!(cols.conid.len(), 2);
        // The surviving row sits at the position of its LAST occurrence.
        // bars[1] (conid=2) precedes bars[2] (the kept conid=1 duplicate),
        // so it comes out first.
        assert_eq!(cols.conid, vec![2, 1]);
        assert_eq!(cols.close, vec!["other", "last"]);
        assert_eq!(cols.volume, vec![Some(99), Some(20)]);
    }

    #[test]
    fn to_columns_treats_different_bar_lengths_as_distinct_keys() {
        let bars = vec![
            bar(1, 1_000, 60, "minute", Some(1)),
            bar(1, 1_000, 300, "five-minute", Some(5)),
        ];

        let cols = to_columns(&bars);

        assert_eq!(cols.conid.len(), 2);
        assert_eq!(cols.bar_length_seconds, vec![60, 300]);
    }

    #[test]
    fn to_columns_handles_empty_input() {
        let cols = to_columns(&[]);
        assert_eq!(cols.conid.len(), 0);
    }
}
