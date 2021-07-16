use alpaca::AlpacaMessage;
use anyhow::{Context, Result};
use futures::StreamExt;
use kafka_settings::consumer;
use lazy_static::lazy_static;
use prometheus::Registry;
use rdkafka::Message;
use serde::Deserialize;
use tracing::subscriber::set_global_default;
use tracing::{info, trace};
use tracing_subscriber::EnvFilter;

mod domain;
mod server;
mod settings;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
}

pub fn register_custom_metrics() {
    domain::orders::register();
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Input {
    Alpaca(AlpacaMessage),
}

pub async fn run_kafka(kafka_settings: &kafka_settings::KafkaSettings) -> Result<()> {
    let consumer = consumer(kafka_settings).context("Failed to create kafka consumer")?;
    while let Some(message) = consumer.stream().next().await {
        let message = message.context("Error from kafka")?;
        if let Some(payload) = message.payload() {
            trace!("Received payload");
            let input: Input =
                serde_json::from_slice(payload).context("Failed to deserialize message")?;
            match input {
                Input::Alpaca(alpaca_message) => {
                    domain::orders::handle_alpaca_message(alpaca_message)
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv::dotenv();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    set_global_default(subscriber).expect("Failed to set subscriber");
    info!("Starting reporting");
    register_custom_metrics();
    let settings = settings::Settings::new().context("Failed to load settings")?;
    tokio::select! {
        _ = run_kafka(&settings.kafka) => Ok(()),
        _ = server::run(settings.webserver.port) => Ok(())
    }
}
