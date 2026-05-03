use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ibkrctl")]
#[command(about = "OAuth-only IBKR CLI for Airflow tasks")]
pub struct Cli {
    #[arg(long, global = true)]
    pub output: Option<PathBuf>,

    #[arg(long, global = true)]
    pub pretty: bool,

    #[arg(long, global = true)]
    pub timeout_seconds: Option<u64>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Oauth {
        #[command(subcommand)]
        command: OauthCommand,
    },
    AuthStatus,
    InitSession {
        #[arg(long, default_value_t = true)]
        compete: bool,
    },
    FetchHistory(FetchHistoryArgs),
    StockConid(StockConidArgs),
    Accounts,
    AccountSummary {
        #[arg(long)]
        account_id: String,
    },
    PortfolioSummary {
        #[arg(long)]
        account_id: String,
    },
    Ledger {
        #[arg(long)]
        account_id: String,
    },
    Positions {
        #[arg(long)]
        account_id: String,
        #[arg(long, default_value_t = 0)]
        page: u32,
    },
    LiveOrders {
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long, default_value_t = true)]
        force: bool,
    },
    Order {
        #[command(subcommand)]
        command: OrderCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum OauthCommand {
    GenerateMaterials {
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        force: bool,
    },
}

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

#[derive(Debug, Args)]
pub struct FetchHistoryArgs {
    #[arg(long)]
    pub conid: String,
    #[arg(long)]
    pub period: String,
    #[arg(long)]
    pub bar: String,
    #[arg(long)]
    pub exchange: Option<String>,
    #[arg(long)]
    pub outside_rth: Option<bool>,
    #[arg(long)]
    pub start_time: Option<String>,
}

#[derive(Debug, Args)]
pub struct StockConidArgs {
    #[arg(long)]
    pub symbol: String,
    #[arg(long)]
    pub exchange: Option<String>,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub default_filtering: bool,
}
