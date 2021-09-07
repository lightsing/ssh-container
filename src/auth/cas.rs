use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CasResponse {
    pub service_response: ServiceResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceResponse {
    AuthenticationSuccess(AuthenticationSuccess),
    AuthenticationFailure(AuthenticationFailure),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSuccess {
    pub attributes: Attributes,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationFailure {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attributes {
    pub id: Vec<String>,
    pub sid: Vec<String>,
}

impl Display for CasResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}
