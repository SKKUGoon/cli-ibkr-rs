use std::path::PathBuf;

use clap::{ArgGroup, Args, Subcommand};

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
#[command(group(
    ArgGroup::new("orders_input")
        .required(true)
        .args(["orders_file", "orders_json"])
))]
#[command(group(
    ArgGroup::new("answers_input")
        .required(true)
        .args(["answers_file", "answers_json"])
))]
pub struct OrderPlaceArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long, conflicts_with = "orders_json")]
    pub orders_file: Option<PathBuf>,
    #[arg(long, conflicts_with = "orders_file")]
    pub orders_json: Option<String>,
    #[arg(long, conflicts_with = "answers_json")]
    pub answers_file: Option<PathBuf>,
    #[arg(long, conflicts_with = "answers_file")]
    pub answers_json: Option<String>,
    #[arg(long, default_value_t = 20)]
    pub max_replies: u32,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("orders_input")
        .required(true)
        .args(["orders_file", "orders_json"])
))]
pub struct OrderWhatifArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long, conflicts_with = "orders_json")]
    pub orders_file: Option<PathBuf>,
    #[arg(long, conflicts_with = "orders_file")]
    pub orders_json: Option<String>,
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
#[command(group(
    ArgGroup::new("order_input")
        .required(true)
        .args(["order_file", "order_json"])
))]
#[command(group(
    ArgGroup::new("answers_input")
        .required(true)
        .args(["answers_file", "answers_json"])
))]
pub struct OrderModifyArgs {
    #[arg(long)]
    pub account_id: String,
    #[arg(long)]
    pub order_id: String,
    #[arg(long, conflicts_with = "order_json")]
    pub order_file: Option<PathBuf>,
    #[arg(long, conflicts_with = "order_file")]
    pub order_json: Option<String>,
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
