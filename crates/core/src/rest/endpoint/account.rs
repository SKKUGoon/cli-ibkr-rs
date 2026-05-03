pub const ACCOUNTS: &str = "portfolio/accounts";

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
