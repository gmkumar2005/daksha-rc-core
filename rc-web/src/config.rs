use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::fmt;

// Handles application configuration,

const CONFIG_FILE_PATH: &str = "./config/Default.toml";
const CONFIG_FILE_PREFIX: &str = "./config/";

#[derive(Clone, Debug, Deserialize)]
pub enum ENV {
    Development,
    Testing,
    Production,
}

impl fmt::Display for ENV {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ENV::Development => write!(f, "Development"),
            ENV::Testing => write!(f, "Testing"),
            ENV::Production => write!(f, "Production"),
        }
    }
}

impl From<&str> for ENV {
    fn from(env: &str) -> Self {
        match env {
            "Testing" => ENV::Testing,
            "Production" => ENV::Production,
            _ => ENV::Development,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
}
#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub database: DatabaseConfig,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "Development".into());
        let builder = Config::builder()
            .add_source(File::with_name(CONFIG_FILE_PATH)) // Load from config file
            .add_source(File::with_name(&format!("{}{}", CONFIG_FILE_PREFIX, env))) // Load from config file
            .add_source(config::Environment::with_prefix("APP")); // Optionally, load from environment variables

        let settings = builder.build()?;
        settings.try_deserialize()
    }
}
