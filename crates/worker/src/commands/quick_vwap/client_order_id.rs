use chrono::NaiveDate;
use ibkr_core::model::order::OrderSide;

pub fn build_vwap_client_order_id(
    prefix: &str,
    symbol: &str,
    order_date: NaiveDate,
    side: OrderSide,
    quantity: f64,
    limit_price: f64,
) -> String {
    format!(
        "{}-{}-{}-{}-new-{}-{}",
        sanitized_component(prefix),
        sanitized_component(&symbol.to_ascii_uppercase()),
        order_date.format("%Y%m%d"),
        side.as_client_order_id_component(),
        compact_quantity(quantity),
        price_in_cents(limit_price),
    )
}

fn sanitized_component(input: &str) -> String {
    let sanitized = input
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect::<String>();
    sanitized.trim_matches('-').to_string()
}

fn compact_quantity(quantity: f64) -> String {
    let formatted = format!("{quantity:.8}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .replace('.', "p")
}

fn price_in_cents(limit_price: f64) -> String {
    format!("{:.0}", limit_price * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_documented_client_order_id_shape() {
        let client_order_id = build_vwap_client_order_id(
            "kappa-k1",
            "tqqq",
            NaiveDate::from_ymd_opt(2026, 7, 23).unwrap(),
            OrderSide::Buy,
            10.0,
            70.0,
        );

        assert_eq!(client_order_id, "kappa-k1-TQQQ-20260723-buy-new-10-7000");
    }
}
