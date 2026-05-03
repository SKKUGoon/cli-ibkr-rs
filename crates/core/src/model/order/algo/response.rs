use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AlgoParamsResponse {
    pub algos: Vec<IbAlgo>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IbAlgo {
    pub name: String,
    pub id: String,
    #[serde(default)]
    pub parameters: Vec<IbAlgoParameter>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct IbAlgoParameter {
    #[serde(rename = "guiRank", skip_serializing_if = "Option::is_none")]
    pub gui_rank: Option<i64>,
    #[serde(rename = "defaultValue", skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(
        rename = "legalStrings",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub legal_strings: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<String>,
    #[serde(rename = "valueClassName", skip_serializing_if = "Option::is_none")]
    pub value_class_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "minValue", skip_serializing_if = "Option::is_none")]
    pub min_value: Option<Value>,
    #[serde(rename = "maxValue", skip_serializing_if = "Option::is_none")]
    pub max_value: Option<Value>,
    #[serde(
        rename = "enabledConditions",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub enabled_conditions: Vec<Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
