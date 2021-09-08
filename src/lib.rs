use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[cfg(feature = "auth")]
pub mod auth;
pub mod config;
pub mod helper;

pub use crate::config::{Config, DaemonConfig, RunnerConfig};
pub use crate::helper::AutoRemoveHashMap;

pub type ChallengeFilter = Arc<AutoRemoveHashMap<String, String>>;
pub type AuthStore = Arc<RwLock<HashMap<String, AuthStatus>>>;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthStatus {
    Assigned,
    Authed(String),
    Failed,
}
