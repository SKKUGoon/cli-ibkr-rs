pub const PORTFOLIO_ACCOUNTS: &str = "portfolio/accounts";
pub const BROKERAGE_ACCOUNTS: &str = "iserver/accounts";
pub const ACCOUNT_PROFIT_AND_LOSS: &str = "iserver/account/pnl/partitioned";

pub fn account_summary(account_id: &str) -> String {
    format!("iserver/account/{account_id}/summary")
}

pub fn portfolio_summary(account_id: &str) -> String {
    format!("portfolio/{account_id}/summary")
}

pub fn ledger(account_id: &str) -> String {
    format!("portfolio/{account_id}/ledger")
}

pub fn positions(account_id: &str, page: u32) -> String {
    format!("portfolio/{account_id}/positions/{page}")
}

pub fn live_positions(account_id: &str) -> String {
    format!("portfolio2/{account_id}/positions")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_live_positions_path_for_account() {
        assert_eq!(live_positions("DU123456"), "portfolio2/DU123456/positions");
    }
}
