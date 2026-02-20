use serde::Deserialize;
use config::{Config, ConfigError, File, Environment};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub port: u16,
    pub log_level: String,
    pub security_boundary: bool,
    pub auto_prune_interval_secs: u64,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .set_default("data_dir", "./api_data")?
            .set_default("port", 3000)?
            .set_default("log_level", "info")?
            .set_default("security_boundary", true)?
            .set_default("auto_prune_interval_secs", 3600)?
            .add_source(File::with_name("config/settings").required(false))
            .add_source(Environment::with_prefix("OMNIFORGE"))
            .build()?;

        s.try_deserialize()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }
        Ok(())
    }
}
