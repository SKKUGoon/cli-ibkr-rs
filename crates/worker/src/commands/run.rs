use ibkr_core::IbkrClient;

use crate::cli::{Cli, Command};
use crate::config::RawConfig;
use crate::error::WorkerError;
use crate::output;

use super::{auth, market_data, oauth, order, portfolio};

pub async fn run(cli: Cli) -> Result<(), WorkerError> {
    if let Command::Oauth { command } = cli.command {
        return oauth::run(command);
    }

    let config = RawConfig::load(cli.timeout_seconds)?.oauth()?;
    let client = IbkrClient::new(config)?;

    let value = match cli.command {
        Command::AuthStatus => auth::auth_status(&client).await?,
        Command::InitSession { compete } => auth::init_session(&client, compete).await?,
        Command::FetchHistory(args) => market_data::fetch_history(&client, args).await?,
        Command::StockConid(args) => market_data::stock_conid(&client, args).await?,
        Command::Accounts => portfolio::accounts(&client).await?,
        Command::AccountSummary { account_id } => {
            portfolio::account_summary(&client, &account_id).await?
        }
        Command::PortfolioSummary { account_id } => {
            portfolio::portfolio_summary(&client, &account_id).await?
        }
        Command::Ledger { account_id } => portfolio::ledger(&client, &account_id).await?,
        Command::Positions { account_id, page } => {
            portfolio::positions(&client, &account_id, page).await?
        }
        Command::LiveOrders { account_id, force } => {
            portfolio::live_orders(&client, account_id.as_deref(), force).await?
        }
        Command::Order { command } => order::run(&client, command).await?,
        Command::Oauth { .. } => unreachable!("oauth handled before client config loading"),
    };

    output::write_json(&value, cli.output.as_deref(), cli.pretty)
}
