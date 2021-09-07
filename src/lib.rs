use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[cfg(feature = "auth")]
pub mod auth;
pub mod config;

pub use crate::config::{Config, DaemonConfig, RunnerConfig};

pub type ChallengeFilter = Arc<RwLock<HashMap<String, String>>>;
pub type AuthStore = Arc<RwLock<HashMap<String, AuthStatus>>>;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthStatus {
    Assigned,
    Authed(String),
    Failed,
}
