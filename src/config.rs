use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct DbConfig {
    pub url: String,
    pub pool: u32
}

impl DbConfig {
    pub fn get() -> Result<Self, ConfigError> {
        let cfg = Config::builder()
            .add_source(Environment::with_prefix("DATABASE"))
            .build()?;

        cfg.try_deserialize()
    }       
}

#[derive(Deserialize, Debug)]
pub struct RedisCfg {
    pub url: String,
    pub pool: usize
}

impl RedisCfg {
    pub fn get() -> Result<Self, ConfigError> {
        let cfg = Config::builder()
            .add_source(Environment::with_prefix("REDIS"))
            .build()?;

        cfg.try_deserialize()
    }
}
