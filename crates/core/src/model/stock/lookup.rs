use serde_json::Value;

use crate::{IbkrError, Result};

use super::{StockConidRequest, StockLookupResult};

pub fn conid_by_symbol(response: &Value, request: &StockConidRequest) -> Result<Value> {
    Ok(serde_json::to_value(stock_by_symbol(response, request)?)?)
}

pub fn stock_by_symbol(response: &Value, request: &StockConidRequest) -> Result<StockLookupResult> {
    let instruments = instruments(response, &request.symbol)?;
    let mut matches = Vec::new();

    for instrument in instruments {
        let name = instrument
            .get("name")
            .and_then(Value::as_str)
            .map(str::to_string);
        let Some(contracts) = instrument.get("contracts").and_then(Value::as_array) else {
            continue;
        };

        for contract in contracts {
            if !matches_default_filter(contract, request.default_filtering) {
                continue;
            }
            if !matches_exchange(contract, request.exchange.as_deref()) {
                continue;
            }
            let Some(conid) = contract.get("conid").and_then(Value::as_i64) else {
                continue;
            };
            let exchange = contract
                .get("exchange")
                .and_then(Value::as_str)
                .map(str::to_string);
            matches.push(StockLookupResult {
                symbol: request.symbol.to_ascii_uppercase(),
                name_: name.clone(),
                conid,
                exchange,
            });
        }
    }

    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(lookup_error(
            request,
            "no contracts matched the requested filters",
        )),
        count => Err(lookup_error(
            request,
            &format!("{count} contracts matched; add --exchange or disable/change filters"),
        )),
    }
}

fn instruments<'a>(response: &'a Value, symbol: &str) -> Result<&'a Vec<Value>> {
    response
        .get(symbol)
        .or_else(|| response.get(symbol.to_uppercase()))
        .and_then(Value::as_array)
        .ok_or_else(|| IbkrError::StockConidLookup {
            symbol: symbol.to_string(),
            message: "symbol not found in IBKR stock response".to_string(),
        })
}

fn matches_default_filter(contract: &Value, default_filtering: bool) -> bool {
    !default_filtering || contract.get("isUS").and_then(Value::as_bool) == Some(true)
}

fn matches_exchange(contract: &Value, exchange: Option<&str>) -> bool {
    exchange.is_none_or(|expected| {
        contract
            .get("exchange")
            .and_then(Value::as_str)
            .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
    })
}

fn lookup_error(request: &StockConidRequest, message: &str) -> IbkrError {
    IbkrError::StockConidLookup {
        symbol: request.symbol.clone(),
        message: message.to_string(),
    }
}
