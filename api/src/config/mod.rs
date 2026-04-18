use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_database_url")]
    pub database_url: String,
}

impl Settings {
    pub fn load() -> Result<Self, config::ConfigError> {
        Config::builder()
            .set_default("host", default_host())?
            .set_default("port", default_port())?
            .set_default("database_url", default_database_url())?
            .add_source(File::with_name("api/config/local").required(false))
            .add_source(Environment::with_prefix("PLAN").separator("__"))
            .build()?
            .try_deserialize()
    }
}

fn default_host() -> String {
    "127.0.0.1".to_owned()
}

fn default_port() -> u16 {
    3001
}

fn default_database_url() -> String {
    "sqlite://data/plan.db?mode=rwc".to_owned()
}
