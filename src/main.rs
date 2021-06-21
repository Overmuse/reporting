use alpaca::AlpacaMessage;
use anyhow::{Context, Result};
use dogstatsd::Client;
use futures::StreamExt;
use kafka_settings::consumer;
use rdkafka::Message;
mod settings;
mod types;
use tracing::subscriber::set_global_default;
use tracing::{debug, info, trace};
use tracing_subscriber::EnvFilter;
use types::ToMetrics;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv::dotenv();
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    set_global_default(subscriber).expect("Failed to set subscriber");
    info!("Starting reporting");
    let settings = settings::Settings::new().context("Failed to load settings")?;
    let consumer = consumer(&settings.kafka).context("Failed to create kafka consumer")?;
    debug!("Creating dogstatsd client");
    let client = Client::new(
        "127.0.0.1:0",
        &format!("{}:8125", settings.app.target_address),
    )
    .await
    .context("Failed to create dogstatsd client")?;
    while let Some(message) = consumer.stream().next().await {
        let message = message.context("Error from kafka")?;
        if let Some(payload) = message.payload() {
            trace!("Received payload");
            let alpaca_message: AlpacaMessage =
                serde_json::from_slice(payload).context("Failed to deserialize alpaca message")?;
            if let AlpacaMessage::TradeUpdates(order_update) = alpaca_message {
                debug!("Received trade update, generating metrics");
                let metrics = order_update
                    .order
                    .to_metrics()
                    .context("Failed to convert order to metrics")?;
                for metric in metrics {
                    client
                        .send(metric)
                        .await
                        .context("Failed to send metrics")?;
                }
            }
        }
    }
    Ok(())
}
