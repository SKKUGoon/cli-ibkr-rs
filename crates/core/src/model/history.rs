#[derive(Debug, Clone)]
pub struct HistoryRequest {
    pub conid: String,
    pub period: String,
    pub bar: String,
    pub exchange: Option<String>,
    pub outside_rth: Option<bool>,
    pub start_time: Option<String>,
}

impl HistoryRequest {
    pub fn params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("conid".to_string(), self.conid.clone()),
            ("period".to_string(), self.period.clone()),
            ("bar".to_string(), self.bar.clone()),
        ];

        push_optional(&mut params, "exchange", &self.exchange);
        push_optional(
            &mut params,
            "outsideRth",
            &self.outside_rth.map(|v| v.to_string()),
        );
        push_optional(&mut params, "startTime", &self.start_time);
        params
    }
}

fn push_optional(params: &mut Vec<(String, String)>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        params.push((key.to_string(), value.clone()));
    }
}
