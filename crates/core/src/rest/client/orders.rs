use serde_json::Value;

use crate::model::order::{AlgoParamsRequest, OrderRequest, PlaceOrdersRequest, ReplyRequest};
use crate::rest::endpoint;
use crate::Result;

use super::IbkrClient;

impl IbkrClient {
    pub async fn place_orders(
        &self,
        account_id: &str,
        request: &PlaceOrdersRequest,
    ) -> Result<Value> {
        self.post_json(
            &endpoint::order::place_orders(account_id),
            &[],
            Some(request),
        )
        .await
    }

    pub async fn whatif_order(
        &self,
        account_id: &str,
        request: &PlaceOrdersRequest,
    ) -> Result<Value> {
        self.post_json(
            &endpoint::order::whatif_orders(account_id),
            &[],
            Some(request),
        )
        .await
    }

    pub async fn reply(&self, reply_id: &str, confirmed: bool) -> Result<Value> {
        let body = ReplyRequest { confirmed };
        self.post_json(&endpoint::order::reply(reply_id), &[], Some(&body))
            .await
    }

    pub async fn cancel_order(&self, account_id: &str, order_id: &str) -> Result<Value> {
        self.delete_json(&endpoint::order::cancel_order(account_id, order_id), &[])
            .await
    }

    pub async fn modify_order(
        &self,
        account_id: &str,
        order_id: &str,
        request: &OrderRequest,
    ) -> Result<Value> {
        self.post_json(
            &endpoint::order::modify_order(account_id, order_id),
            &[],
            Some(request),
        )
        .await
    }

    pub async fn order_status(&self, order_id: &str) -> Result<Value> {
        self.get_json(&endpoint::order::order_status(order_id), &[])
            .await
    }

    pub async fn order_algos(
        &self,
        conid: &str,
        algos: &[String],
        add_description: bool,
        add_params: bool,
    ) -> Result<Value> {
        let request = AlgoParamsRequest::new(conid, algos.to_vec(), add_description, add_params)?;
        let params = request.params();
        self.get_json(&endpoint::order::algos(request.conid()), &params)
            .await
    }
}
