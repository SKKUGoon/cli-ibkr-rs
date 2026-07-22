mod account;
mod order;
mod quick_vwap;
#[cfg(test)]
mod tests;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

pub use account::PositionsLiveArgs;
pub use order::{
    OrderAlgosArgs, OrderCancelArgs, OrderCommand, OrderFeePlanArgs, OrderModifyArgs,
    OrderPlaceArgs, OrderReplyArgs, OrderStatusArgs, OrderWhatifArgs,
};
pub use quick_vwap::{QuickOrderSide, QuickVwapOrderArgs};

#[derive(Debug, Parser)]
#[command(name = "ibkrctl")]
#[command(about = "OAuth-only IBKR CLI for Airflow tasks")]
#[command(
    after_help = "--------------------\nINTERACTIVE UTILITIES\n  quick-vwap-order  Prompt for and submit one manual VWAP limit order"
)]
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
    /// Initialize and list accounts available to portfolio endpoints.
    Accounts,
    /// Initialize and list accounts available to IServer trading endpoints.
    BrokerageAccounts,
    /// Return P&L for the currently selected account and its models.
    AccountPnl,
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
    /// Return uncached, near-real-time positions through the REST API.
    PositionsLive(PositionsLiveArgs),
    Trades(TradesArgs),
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
    #[command(
        hide = true,
        about = "Interactively build, review, and submit one manual VWAP limit order"
    )]
    QuickVwapOrder(QuickVwapOrderArgs),
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

#[derive(Debug, Args)]
pub struct TradesArgs {
    #[arg(long)]
    pub account_id: Option<String>,
    #[arg(long)]
    pub days: Option<u8>,
}
