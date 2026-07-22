use clap::{CommandFactory, Parser};

use super::{Cli, Command};

#[test]
fn trades_accepts_no_optional_args() {
    let cli = Cli::try_parse_from(["ibkrctl", "trades"]).expect("trades should parse");

    match cli.command {
        Command::Trades(arguments) => {
            assert_eq!(arguments.account_id, None);
            assert_eq!(arguments.days, None);
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
        Command::Trades(arguments) => {
            assert_eq!(arguments.account_id.as_deref(), Some("DU123456"));
            assert_eq!(arguments.days, Some(7));
        }
        other => panic!("expected trades command, got {other:?}"),
    }
}

#[test]
fn trades_rejects_non_numeric_days() {
    let parse_error = Cli::try_parse_from(["ibkrctl", "trades", "--days", "seven"])
        .expect_err("non-numeric days should fail");

    assert!(parse_error.to_string().contains("invalid value"));
}

#[test]
fn positions_live_accepts_optional_filters() {
    let cli = Cli::try_parse_from([
        "ibkrctl",
        "positions-live",
        "--account-id",
        "DU123456",
        "--model",
        "growth",
        "--sort",
        "marketValue",
        "--direction",
        "d",
    ])
    .expect("live position filters should parse");

    match cli.command {
        Command::PositionsLive(arguments) => {
            assert_eq!(arguments.account_id, "DU123456");
            assert_eq!(arguments.model.as_deref(), Some("growth"));
            assert_eq!(arguments.direction.as_deref(), Some("d"));
        }
        other => panic!("expected positions-live command, got {other:?}"),
    }
}

#[test]
fn help_separates_interactive_utility() {
    let help = Cli::command().render_long_help().to_string();

    assert!(help.contains("INTERACTIVE UTILITIES"));
    assert!(help.contains("quick-vwap-order"));
}
