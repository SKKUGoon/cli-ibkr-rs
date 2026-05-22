mod order;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

pub use order::{
    OrderAlgosArgs, OrderCancelArgs, OrderCommand, OrderFeePlanArgs, OrderModifyArgs,
    OrderPlaceArgs, OrderReplyArgs, OrderStatusArgs, OrderWhatifArgs,
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

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;

    #[test]
    fn trades_accepts_no_optional_args() {
        let cli = Cli::try_parse_from(["ibkrctl", "trades"]).expect("trades should parse");

        match cli.command {
            Command::Trades(args) => {
                assert_eq!(args.account_id, None);
                assert_eq!(args.days, None);
            }
            other => panic!("expected trades command, got {other:?}"),
        }
    }

    #[test]
    fn trades_accepts_account_id_and_days() {
        let cli = Cli::try_parse_from([
            "ibkrctl",
            "trades",
            "--account-id",
            "DU123456",
            "--days",
            "7",
        ])
        .expect("trades with filters should parse");

        match cli.command {
            Command::Trades(args) => {
                assert_eq!(args.account_id.as_deref(), Some("DU123456"));
                assert_eq!(args.days, Some(7));
            }
            other => panic!("expected trades command, got {other:?}"),
        }
    }

    #[test]
    fn trades_rejects_non_numeric_days() {
        let err = Cli::try_parse_from(["ibkrctl", "trades", "--days", "seven"])
            .expect_err("non-numeric days should fail");

        assert!(err.to_string().contains("invalid value"));
    }
}
