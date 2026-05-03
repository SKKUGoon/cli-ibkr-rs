use serde_json::{json, Value};

use crate::Result;

use super::IbkrClient;
use crate::rest::endpoint;

impl IbkrClient {
    pub async fn auth_status(&self) -> Result<Value> {
        self.post_json(endpoint::AUTH_STATUS, &[], None::<&()>)
            .await
    }

    pub async fn init_session(&self, compete: bool) -> Result<Value> {
        let body = json!({ "publish": true, "compete": compete });
        self.post_json(endpoint::INIT_SESSION, &[], Some(&body))
            .await
    }
}
