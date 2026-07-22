use clap::{Args, ValueEnum};
use ibkr_core::model::order::OrderSide;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum QuickOrderSide {
    Buy,
    Sell,
}

impl From<QuickOrderSide> for OrderSide {
    fn from(side: QuickOrderSide) -> Self {
        match side {
            QuickOrderSide::Buy => Self::Buy,
            QuickOrderSide::Sell => Self::Sell,
        }
    }
}

#[derive(Debug, Args)]
pub struct QuickVwapOrderArgs {
    #[arg(long)]
    pub account_id: Option<String>,
    #[arg(long)]
    pub ticker: Option<String>,
    #[arg(long)]
    pub exchange: Option<String>,
    #[arg(long, value_enum)]
    pub side: Option<QuickOrderSide>,
    #[arg(long)]
    pub quantity: Option<f64>,
    #[arg(long)]
    pub limit_price: Option<f64>,
    #[arg(long)]
    pub start_time: Option<String>,
    #[arg(long)]
    pub end_time: Option<String>,
    #[arg(long)]
    pub max_percent_volume: Option<String>,
    #[arg(long)]
    pub client_order_id_prefix: Option<String>,
    #[arg(long, default_value_t = 20)]
    pub max_replies: u32,
}
