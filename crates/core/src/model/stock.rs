use serde_json::{json, Value};

use crate::{IbkrError, Result};

#[derive(Debug, Clone)]
pub struct StockConidRequest {
    pub symbol: String,
    pub exchange: Option<String>,
    pub default_filtering: bool,
}

impl StockConidRequest {
    pub fn params(&self) -> Vec<(String, String)> {
        vec![("symbols".to_string(), self.symbol.clone())]
    }
}

pub fn conid_by_symbol(response: &Value, request: &StockConidRequest) -> Result<Value> {
    let instruments = instruments(response, &request.symbol)?;
    let mut matches = Vec::new();

    for instrument in instruments {
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
            let Some(conid) = contract.get("conid") else {
                continue;
            };
            matches.push(conid.clone());
        }
    }

    match matches.len() {
        1 => Ok(json!({ request.symbol.clone(): matches.remove(0) })),
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn one_us_contract_returns_conid() {
        let response = json!({
            "AAPL": [{
                "contracts": [
                    {"conid": 265598, "exchange": "NASDAQ", "isUS": true}
                ]
            }]
        });
        let request = request("AAPL", None, true);

        let value = conid_by_symbol(&response, &request).expect("lookup should succeed");

        assert_eq!(value, json!({"AAPL": 265598}));
    }

    #[test]
    fn no_matching_contract_returns_error() {
        let response = json!({
            "AAPL": [{
                "contracts": [
                    {"conid": 123, "exchange": "MEXI", "isUS": false}
                ]
            }]
        });
        let request = request("AAPL", None, true);

        let err = conid_by_symbol(&response, &request).expect_err("lookup should fail");

        assert!(err.to_string().contains("no contracts matched"));
    }

    #[test]
    fn multiple_matching_contracts_returns_ambiguity_error() {
        let response = json!({
            "AAPL": [{
                "contracts": [
                    {"conid": 265598, "exchange": "NASDAQ", "isUS": true},
                    {"conid": 111, "exchange": "NYSE", "isUS": true}
                ]
            }]
        });
        let request = request("AAPL", None, true);

        let err = conid_by_symbol(&response, &request).expect_err("lookup should fail");

        assert!(err.to_string().contains("2 contracts matched"));
    }

    #[test]
    fn exchange_narrows_ambiguous_result() {
        let response = json!({
            "AAPL": [{
                "contracts": [
                    {"conid": 265598, "exchange": "NASDAQ", "isUS": true},
                    {"conid": 111, "exchange": "NYSE", "isUS": true}
                ]
            }]
        });
        let request = request("AAPL", Some("nasdaq"), true);

        let value = conid_by_symbol(&response, &request).expect("lookup should succeed");

        assert_eq!(value, json!({"AAPL": 265598}));
    }

    #[test]
    fn default_filtering_false_disables_us_filter() {
        let response = json!({
            "AAPL": [{
                "contracts": [
                    {"conid": 555, "exchange": "MEXI", "isUS": false}
                ]
            }]
        });
        let request = request("AAPL", None, false);

        let value = conid_by_symbol(&response, &request).expect("lookup should succeed");

        assert_eq!(value, json!({"AAPL": 555}));
    }

    fn request(symbol: &str, exchange: Option<&str>, default_filtering: bool) -> StockConidRequest {
        StockConidRequest {
            symbol: symbol.to_string(),
            exchange: exchange.map(str::to_string),
            default_filtering,
        }
    }
}
