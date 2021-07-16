use config::{Config, ConfigError, Environment};
use kafka_settings::KafkaSettings;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct WebServerSettings {
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub webserver: WebServerSettings,
    pub kafka: KafkaSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(Environment::new().separator("__"))?;
        s.try_into()
    }
}
