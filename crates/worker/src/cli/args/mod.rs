mod order;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

pub use order::{
    OrderAlgosArgs, OrderCancelArgs, OrderCommand, OrderModifyArgs, OrderPlaceArgs, OrderReplyArgs,
    OrderStatusArgs, OrderWhatifArgs,
};

#[derive(Debug, Parser)]
#[command(name = "ibkrctl")]
#[command(about = "OAuth-only IBKR CLI for Airflow tasks")]
pub struct Cli {
    #[arg(long, global = true)]
    pub env_file: Option<PathBuf>,

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
    Env,
    InitSession {
        #[arg(long, default_value_t = true)]
        compete: bool,
    },
    Tickle,
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
