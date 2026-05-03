use crate::{IbkrError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgoParamsRequest {
    conid: String,
    algos: Vec<String>,
    add_description: bool,
    add_params: bool,
}

impl AlgoParamsRequest {
    pub fn new(
        conid: impl Into<String>,
        algos: Vec<String>,
        add_description: bool,
        add_params: bool,
    ) -> Result<Self> {
        let conid = conid.into().trim().to_string();
        if conid.is_empty() {
            return invalid("conid cannot be empty");
        }
        if algos.len() > 8 {
            return invalid("algos cannot contain more than 8 ids");
        }
        Ok(Self {
            conid,
            algos,
            add_description,
            add_params,
        })
    }

    pub fn conid(&self) -> &str {
        &self.conid
    }

    pub fn algos(&self) -> &[String] {
        &self.algos
    }

    pub fn add_description(&self) -> bool {
        self.add_description
    }

    pub fn add_params(&self) -> bool {
        self.add_params
    }

    pub fn params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if !self.algos.is_empty() {
            params.push(("algos".to_string(), self.algos.join(";")));
        }
        params.push((
            "addDescription".to_string(),
            bool_query_value(self.add_description).to_string(),
        ));
        params.push((
            "addParams".to_string(),
            bool_query_value(self.add_params).to_string(),
        ));
        params
    }
}

fn bool_query_value(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}

fn invalid<T>(message: &str) -> Result<T> {
    Err(IbkrError::InvalidOrderRequest(message.to_string()))
}
