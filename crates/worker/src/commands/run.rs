use ibkr_core::IbkrClient;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tokio::time;

use crate::cli::{Cli, Command, OrderCommand};
use crate::config::RawConfig;
use crate::error::WorkerError;
use crate::output;

use super::{environment, market_data, oauth, order};

pub async fn run(cli: Cli) -> Result<(), WorkerError> {
    if let Command::Oauth { command } = cli.command {
        return oauth::run(command);
    }
    if let Command::Env = cli.command {
        return output::write_text(&environment::render(), cli.output.as_deref());
    }
    if let Command::Order {
        command: OrderCommand::FeePlan(args),
    } = cli.command
    {
        let value = order::fee_plan(args).await?;
        return output::write_json(&value, cli.output.as_deref(), cli.pretty);
    }

    let raw_config = RawConfig::load(cli.timeout_seconds)?;
    let database = raw_config.database.clone();
    let config = raw_config.oauth()?;
    let client = IbkrClient::new(config)?;

    let value = match cli.command {
        Command::AuthStatus => client.auth_status().await?,
        Command::Env => unreachable!("env handled before client config loading"),
        Command::InitSession { compete } => client.init_session(compete).await?,
        Command::Tickle => client.tickle().await?,
        Command::FetchHistory(args) => {
            let pool = connect_database(database.as_deref()).await;
            market_data::fetch_history(pool.as_ref(), &client, args).await?
        }
        Command::StockConid(args) => {
            let pool = connect_database(database.as_deref()).await;
            market_data::stock_conid_cached(pool.as_ref(), &client, args).await?
        }
        Command::Accounts => client.accounts().await?,
        Command::AccountSummary { account_id } => client.account_summary(&account_id).await?,
        Command::PortfolioSummary { account_id } => client.portfolio_summary(&account_id).await?,
        Command::Ledger { account_id } => client.ledger(&account_id).await?,
        Command::Positions { account_id, page } => client.positions(&account_id, page).await?,
        Command::Trades(args) => client.trades(args.account_id.as_deref(), args.days).await?,
        Command::LiveOrders { account_id, force } => {
            client.live_orders(account_id.as_deref(), force).await?
        }
        Command::Order { command } => order::run(&client, command).await?,
        Command::Oauth { .. } => unreachable!("oauth handled before client config loading"),
    };

    output::write_json(&value, cli.output.as_deref(), cli.pretty)
}

async fn connect_database(database: Option<&str>) -> Option<PgPool> {
    let Some(database) = database else {
        tracing::warn!("IBKR_DATABASE is not set; skipping local database lookup or persistence");
        return None;
    };

    let connect = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(15))
        .connect(database);

    match time::timeout(Duration::from_secs(15), connect).await {
        Ok(Ok(pool)) => Some(pool),
        Ok(Err(err)) => {
            tracing::warn!(
                error = %err,
                "IBKR_DATABASE is not connectable; skipping local database lookup or persistence"
            );
            None
        }
        Err(_) => {
            tracing::warn!(
                "IBKR_DATABASE connection timed out after 15 seconds; skipping local database lookup or persistence"
            );
            None
        }
    }
}
