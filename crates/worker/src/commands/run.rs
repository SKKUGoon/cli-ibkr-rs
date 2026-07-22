use ibkr_core::IbkrClient;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tokio::time;

use crate::cli::{Cli, Command, OrderCommand};
use crate::config::RawConfig;
use crate::error::WorkerError;
use crate::output;

use super::{environment, market_data, oauth, order, quick_vwap};

pub async fn execute_cli_command(cli: Cli) -> Result<(), WorkerError> {
    if let Command::Oauth { command } = cli.command {
        return oauth::execute_oauth_command(command);
    }
    if let Command::Env = cli.command {
        return output::write_text(
            &environment::render_environment_report(),
            cli.output.as_deref(),
        );
    }
    if let Command::Order {
        command: OrderCommand::FeePlan(args),
    } = cli.command
    {
        let fee_plan_response = order::calculate_order_fee_plan(args).await?;
        return output::write_json(&fee_plan_response, cli.output.as_deref(), cli.pretty);
    }

    let raw_config = RawConfig::load(cli.timeout_seconds)?;
    let database = raw_config.database.clone();
    let config = raw_config.oauth()?;
    let client = IbkrClient::new(config)?;

    let command_response = match cli.command {
        Command::AuthStatus => client.auth_status().await?,
        Command::Env => unreachable!("env handled before client config loading"),
        Command::InitSession { compete } => client.init_session(compete).await?,
        Command::Tickle => client.tickle().await?,
        Command::FetchHistory(args) => {
            let database_pool = connect_optional_database(database.as_deref()).await;
            market_data::fetch_history(database_pool.as_ref(), &client, args).await?
        }
        Command::StockConid(args) => {
            let database_pool = connect_optional_database(database.as_deref()).await;
            market_data::stock_conid_cached(database_pool.as_ref(), &client, args).await?
        }
        Command::Accounts => client.portfolio_accounts().await?,
        Command::BrokerageAccounts => client.brokerage_accounts().await?,
        Command::AccountPnl => client.account_profit_and_loss().await?,
        Command::AccountSummary { account_id } => client.account_summary(&account_id).await?,
        Command::PortfolioSummary { account_id } => client.portfolio_summary(&account_id).await?,
        Command::Ledger { account_id } => client.ledger(&account_id).await?,
        Command::Positions { account_id, page } => client.positions(&account_id, page).await?,
        Command::PositionsLive(args) => {
            client
                .live_positions(
                    &args.account_id,
                    args.model.as_deref(),
                    args.sort.as_deref(),
                    args.direction.as_deref(),
                )
                .await?
        }
        Command::Trades(args) => client.trades(args.account_id.as_deref(), args.days).await?,
        Command::LiveOrders { account_id, force } => {
            client.live_orders(account_id.as_deref(), force).await?
        }
        Command::Order { command } => order::execute_order_command(&client, command).await?,
        Command::QuickVwapOrder(arguments) => {
            quick_vwap::prompt_and_submit_vwap_order(&client, arguments).await?
        }
        Command::Oauth { .. } => unreachable!("oauth handled before client config loading"),
    };

    output::write_json(&command_response, cli.output.as_deref(), cli.pretty)
}

async fn connect_optional_database(database: Option<&str>) -> Option<PgPool> {
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
