use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum OrderCommand {
    Algos(OrderAlgosArgs),
    Place(OrderPlaceArgs),
    Whatif(OrderWhatifArgs),
    Reply(OrderReplyArgs),
    Cancel(OrderCancelArgs),
    Modify(OrderModifyArgs),
    Status(OrderStatusArgs),
    FeePlan(OrderFeePlanArgs),
}

#[derive(Debug, Args)]
pub struct OrderAlgosArgs {
    #[arg(long)]
    pub conid: String,
    #[arg(long = "algo")]
    pub algos: Vec<String>,
    #[arg(long)]
    pub add_description: bool,
    #[arg(long)]
    pub add_params: bool,
}

#[derive(Debug, Args)]
pub struct OrderPlaceArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long)]
    pub orders_json: String,
    #[arg(long, conflicts_with = "answers_json")]
    pub answers_file: Option<PathBuf>,
    #[arg(long, conflicts_with = "answers_file")]
    pub answers_json: Option<String>,
    #[arg(long, default_value_t = 20)]
    pub max_replies: u32,
}

#[derive(Debug, Args)]
pub struct OrderWhatifArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long)]
    pub orders_json: String,
}

#[derive(Debug, Args)]
pub struct OrderReplyArgs {
    #[arg(long)]
    pub reply_id: String,
    #[arg(long)]
    pub confirmed: bool,
}

#[derive(Debug, Args)]
pub struct OrderCancelArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long)]
    pub order_id: String,
}

#[derive(Debug, Args)]
pub struct OrderModifyArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long)]
    pub order_id: String,
    #[arg(long)]
    pub order_json: String,
    #[arg(long, conflicts_with = "answers_json")]
    pub answers_file: Option<PathBuf>,
    #[arg(long, conflicts_with = "answers_file")]
    pub answers_json: Option<String>,
    #[arg(long, default_value_t = 20)]
    pub max_replies: u32,
}

#[derive(Debug, Args)]
pub struct OrderStatusArgs {
    #[arg(long)]
    pub order_id: String,
}

#[derive(Debug, Args)]
pub struct OrderFeePlanArgs {
    #[arg(long)]
    pub orders_json: String,
}
