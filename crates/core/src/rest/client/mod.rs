mod auth;
mod market_data;
mod orders;
mod portfolio;
mod transport;

use reqwest::Client;

use crate::oauth::{LiveSessionProvider, OAuthConfig};
use crate::Result;

pub struct IbkrClient {
    http: Client,
    config: OAuthConfig,
    live_sessions: LiveSessionProvider,
}

impl IbkrClient {
    pub fn new(config: OAuthConfig) -> Result<Self> {
        let http = Client::builder().timeout(config.timeout).build()?;
        let live_sessions = LiveSessionProvider::new(http.clone(), config.clone())?;
        Ok(Self {
            http,
            config,
            live_sessions,
        })
    }
}
