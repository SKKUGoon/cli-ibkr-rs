use std::env;

use dialoguer::{Input, Select};
use ibkr_core::model::order::OrderSide;

use crate::cli::{QuickOrderSide, QuickVwapOrderArgs};
use crate::error::WorkerError;

pub struct QuickVwapSelections {
    pub account_id: String,
    pub ticker: String,
    pub exchange: Option<String>,
    pub side: OrderSide,
    pub quantity: f64,
    pub limit_price: f64,
    pub start_time: String,
    pub end_time: String,
    pub max_percent_volume: String,
    pub client_order_id_prefix: String,
    pub max_replies: u32,
}

pub fn prompt_for_vwap_order(
    arguments: QuickVwapOrderArgs,
) -> Result<QuickVwapSelections, WorkerError> {
    let account_default = arguments.account_id.or_else(configured_account_id);
    let account_id = required_text("Account ID", account_default)?;
    let ticker = required_text("Ticker", arguments.ticker)?.to_ascii_uppercase();
    let exchange = optional_text("Exchange (blank for automatic)", arguments.exchange)?;
    let side = selected_side(arguments.side)?;
    let quantity = positive_number("Quantity", arguments.quantity)?;
    let limit_price = positive_number("Limit price", arguments.limit_price)?;
    let start_time = required_text(
        "VWAP start time",
        arguments
            .start_time
            .or_else(|| Some("15:30:00 US/Eastern".to_string())),
    )?;
    let end_time = required_text(
        "VWAP end time",
        arguments
            .end_time
            .or_else(|| Some("16:00:00 US/Eastern".to_string())),
    )?;
    let max_percent_volume = required_text(
        "Maximum percent volume",
        arguments
            .max_percent_volume
            .or_else(|| Some("0.1".to_string())),
    )?;
    let prefix_default = arguments
        .client_order_id_prefix
        .or_else(|| env::var("IBKR_QUICK_ORDER_PREFIX").ok())
        .or_else(|| Some("quick-vwap".to_string()));
    let client_order_id_prefix = required_text("Client order ID prefix", prefix_default)?;

    Ok(QuickVwapSelections {
        account_id,
        ticker,
        exchange,
        side,
        quantity,
        limit_price,
        start_time,
        end_time,
        max_percent_volume,
        client_order_id_prefix,
        max_replies: arguments.max_replies,
    })
}

fn configured_account_id() -> Option<String> {
    env::var("IBKR_ACCOUNT_ID")
        .ok()
        .or_else(|| env::var("IBKRCTL_ACCOUNT_ID").ok())
}

fn required_text(prompt: &str, initial: Option<String>) -> Result<String, dialoguer::Error> {
    let mut input = Input::<String>::new().with_prompt(prompt);
    if let Some(initial) = initial {
        input = input.with_initial_text(initial);
    }
    input = input.validate_with(|value: &String| {
        (!value.trim().is_empty())
            .then_some(())
            .ok_or_else(|| "value cannot be empty".to_string())
    });
    input.interact_text().map(|value| value.trim().to_string())
}

fn optional_text(
    prompt: &str,
    initial: Option<String>,
) -> Result<Option<String>, dialoguer::Error> {
    let mut input = Input::<String>::new().with_prompt(prompt).allow_empty(true);
    if let Some(initial) = initial {
        input = input.with_initial_text(initial);
    }
    input.interact_text().map(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn selected_side(initial: Option<QuickOrderSide>) -> Result<OrderSide, dialoguer::Error> {
    if let Some(side) = initial {
        return Ok(side.into());
    }
    let selected_index = Select::new()
        .with_prompt("Side")
        .items(["BUY", "SELL"])
        .default(0)
        .interact()?;
    Ok(if selected_index == 0 {
        OrderSide::Buy
    } else {
        OrderSide::Sell
    })
}

fn positive_number(prompt: &str, initial: Option<f64>) -> Result<f64, dialoguer::Error> {
    let mut input = Input::<f64>::new().with_prompt(prompt);
    if let Some(initial) = initial {
        input = input.with_initial_text(initial.to_string());
    }
    input = input.validate_with(|value: &f64| {
        (value.is_finite() && *value > 0.0)
            .then_some(())
            .ok_or_else(|| "value must be a positive number".to_string())
    });
    input.interact_text()
}
