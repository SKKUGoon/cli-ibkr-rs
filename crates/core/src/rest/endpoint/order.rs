pub const LIVE_ORDERS: &str = "iserver/account/orders";
pub const TRADES: &str = "iserver/account/trades/";

pub fn place_orders(account_id: &str) -> String {
    format!("iserver/account/{account_id}/orders")
}

pub fn whatif_orders(account_id: &str) -> String {
    format!("iserver/account/{account_id}/orders/whatif")
}

pub fn reply(reply_id: &str) -> String {
    format!("iserver/reply/{reply_id}")
}

pub fn cancel_order(account_id: &str, order_id: &str) -> String {
    format!("iserver/account/{account_id}/order/{order_id}")
}

pub fn modify_order(account_id: &str, order_id: &str) -> String {
    format!("iserver/account/{account_id}/order/{order_id}")
}

pub fn order_status(order_id: &str) -> String {
    format!("iserver/account/order/status/{order_id}")
}

pub fn algos(conid: &str) -> String {
    format!("iserver/contract/{conid}/algos")
}
