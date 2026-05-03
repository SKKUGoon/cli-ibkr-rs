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
    pub orders_file: PathBuf,
    #[arg(long)]
    pub answers_file: PathBuf,
    #[arg(long, default_value_t = 20)]
    pub max_replies: u32,
}

#[derive(Debug, Args)]
pub struct OrderWhatifArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long)]
    pub orders_file: PathBuf,
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
    pub order_file: PathBuf,
    #[arg(long)]
    pub answers_file: PathBuf,
    #[arg(long, default_value_t = 20)]
    pub max_replies: u32,
}

#[derive(Debug, Args)]
pub struct OrderStatusArgs {
    #[arg(long)]
    pub order_id: String,
}
