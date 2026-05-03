use serde_json::json;

use super::*;

#[test]
fn one_us_contract_returns_conid() {
    let response = json!({
        "AAPL": [{
            "name": "APPLE INC",
            "contracts": [
                {"conid": 265598, "exchange": "NASDAQ", "isUS": true}
            ]
        }]
    });
    let request = request("AAPL", None, true);

    let value = conid_by_symbol(&response, &request).expect("lookup should succeed");

    assert_eq!(
        value,
        json!({
            "symbol": "AAPL",
            "name_": "APPLE INC",
            "conid": 265598,
            "exchange": "NASDAQ"
        })
    );
}

#[test]
fn stock_lookup_returns_name_conid_and_exchange() {
    let response = json!({
        "AAPL": [{
            "name": "APPLE INC",
            "chineseName": "苹果公司",
            "assetClass": "STK",
            "contracts": [
                {"conid": 265598, "exchange": "NASDAQ", "isUS": true}
            ]
        }]
    });
    let request = request("aapl", None, true);

    let value = stock_by_symbol(&response, &request).expect("lookup should succeed");

    assert_eq!(
        value,
        StockLookupResult {
            symbol: "AAPL".to_string(),
            name_: Some("APPLE INC".to_string()),
            conid: 265598,
            exchange: Some("NASDAQ".to_string()),
        }
    );
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

    assert_eq!(value["conid"], 265598);
    assert_eq!(value["exchange"], "NASDAQ");
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

    assert_eq!(value["conid"], 555);
    assert_eq!(value["exchange"], "MEXI");
}

fn request(symbol: &str, exchange: Option<&str>, default_filtering: bool) -> StockConidRequest {
    StockConidRequest {
        symbol: symbol.to_string(),
        exchange: exchange.map(str::to_string),
        default_filtering,
    }
}
