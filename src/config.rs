use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("i/o error occurred when reading config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("error occurred when parsing config file: {0}")]
    Toml(#[from] toml::de::Error),
}

type Result<T, E = ConfigError> = std::result::Result<T, E>;

const DEFAULT_DAEMON_CONFIG_FILE: &str = "/etc/ssh-containerd.conf";

pub trait Config: Sized + DeserializeOwned {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(toml::from_slice(fs::read(path)?.as_slice())?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DaemonConfig {
    server: DaemonServerConfig,
    auth: AuthConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunnerConfig {
    controller: ControllerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DaemonServerConfig {
    #[serde(default)]
    bind: String,
    #[serde(default)]
    hostname: String,
    #[serde(default)]
    scheme: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ControllerConfig {
    #[serde(default)]
    path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    cas: Option<CasConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CasConfig {
    endpoint: String,
    callback: String,
}

impl DaemonConfig {
    pub fn server(&self) -> &DaemonServerConfig {
        &self.server
    }

    pub fn auth(&self) -> &AuthConfig {
        &self.auth
    }
}

impl Config for DaemonConfig {}

impl RunnerConfig {
    pub fn controller(&self) -> &ControllerConfig {
        &self.controller
    }
}

impl Config for RunnerConfig {}

impl DaemonServerConfig {
    pub fn bind(&self) -> &str {
        &self.bind
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn scheme(&self) -> &str {
        &self.scheme
    }
}

impl ControllerConfig {
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl AuthConfig {
    pub fn cas(&self) -> Option<&CasConfig> {
        self.cas.as_ref()
    }
}

impl CasConfig {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn gen_callback(&self, challenge: &str) -> String {
        format!("{}?challenge={}", self.callback, challenge)
    }

    pub fn gen_auth_url(&self, challenge: &str) -> String {
        format!(
            "{}/login?service={}",
            self.endpoint,
            urlencoding::encode(&*self.gen_callback(challenge))
        )
    }

    pub fn gen_request_url(&self, challenge: &str, ticket: &str) -> String {
        format!(
            "{}/p3/serviceValidate?service={}&format=json&ticket={}",
            self.endpoint,
            urlencoding::encode(&*self.gen_callback(challenge)),
            ticket
        )
    }
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self::new(DEFAULT_DAEMON_CONFIG_FILE).expect("cannot load default config file")
    }
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            path: "/var/run/ssh-containerd.sock".to_string(),
        }
    }
}

impl Default for DaemonServerConfig {
    fn default() -> Self {
        Self {
            bind: "[::]:80".to_string(),
            hostname: "localhost".to_string(),
            scheme: "http".to_string(),
        }
    }
}
