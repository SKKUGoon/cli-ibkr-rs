use super::*;
use crate::IbkrError;

#[test]
fn serializes_ibkr_field_names() {
    let request = parse_orders_json(
        r#"{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","acct_id":"DU123","coid":"x","outside_rth":true}"#,
    )
    .unwrap();

    let value = serde_json::to_value(request).unwrap();
    let order = &value["orders"][0];
    assert_eq!(order["orderType"], "LMT");
    assert_eq!(order["acctId"], "DU123");
    assert_eq!(order["cOID"], "x");
    assert_eq!(order["outsideRTH"], true);
}

#[test]
fn serializes_algo_order_fields() {
    let request = parse_orders_json(
        r#"{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","strategy":"Vwap","strategy_parameters":{"maxPctVol":"0.1"}}"#,
    )
    .unwrap();

    let value = serde_json::to_value(request).unwrap();
    let order = &value["orders"][0];
    assert_eq!(order["strategy"], "Vwap");
    assert_eq!(order["strategyParameters"]["maxPctVol"], "0.1");
}

#[test]
fn computes_automatic_fee_plan_from_notional() {
    let tiered = parse_order_json(r#"{"quantity":100,"price":100}"#).unwrap();
    assert_eq!(tiered.trading_notional(), Some(10_000.0));
    assert_eq!(tiered.automatic_fee_plan(), Some(OrderFeePlan::Tiered));

    let fixed = parse_order_json(r#"{"quantity":100,"price":100.01}"#).unwrap();
    assert_eq!(fixed.trading_notional(), Some(10_001.0));
    assert_eq!(fixed.automatic_fee_plan(), Some(OrderFeePlan::Fixed));
}

#[test]
fn omits_automatic_fee_plan_without_quantity_or_limit_price() {
    let request = parse_order_json(r#"{"quantity":100,"orderType":"MKT"}"#).unwrap();
    assert_eq!(request.trading_notional(), None);
    assert_eq!(request.automatic_fee_plan(), None);
}

#[test]
fn parses_single_object_and_array() {
    let single = parse_orders_json(r#"{"conid":1,"side":"BUY"}"#).unwrap();
    assert_eq!(single.orders.len(), 1);

    let multiple = parse_orders_json(r#"[{"conid":1},{"conid":2}]"#).unwrap();
    assert_eq!(multiple.orders.len(), 2);
}

#[test]
fn rejects_conid_with_conidex() {
    let err = parse_orders_json(r#"{"conid":1,"conidex":"1@SMART"}"#).unwrap_err();
    assert!(matches!(err, IbkrError::InvalidOrderRequest(_)));
    assert!(err.to_string().contains("conid and conidex"));
}

#[test]
fn rejects_conflicting_quantity_fields() {
    let cases = [
        r#"{"quantity":1,"cashQty":200}"#,
        r#"{"quantity":1,"fxQty":100}"#,
        r#"{"cashQty":200,"fxQty":100}"#,
    ];

    for case in cases {
        let err = parse_orders_json(case).unwrap_err();
        assert!(matches!(err, IbkrError::InvalidOrderRequest(_)));
    }
}

#[test]
fn rejects_strategy_parameters_without_strategy() {
    let err =
        parse_orders_json(r#"{"conid":1,"strategyParameters":{"maxPctVol":"0.1"}}"#).unwrap_err();

    assert!(matches!(err, IbkrError::InvalidOrderRequest(_)));
    assert!(err.to_string().contains("without strategy"));
}
