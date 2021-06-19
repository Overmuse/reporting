use alpaca::AlpacaMessage;
use anyhow::Result;
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
    let settings = settings::Settings::new()?;
    let consumer = consumer(&settings.kafka)?;
    debug!("Creating dogstatsd client");
    let client = Client::new("0.0.0.0:0", &settings.app.target_address).await?;
    while let Some(message) = consumer.stream().next().await {
        let message = message?;
        if let Some(payload) = message.payload() {
            trace!("Received payload");
            let alpaca_message: AlpacaMessage = serde_json::from_slice(payload)?;
            if let AlpacaMessage::TradeUpdates(order_update) = alpaca_message {
                debug!("Received trade update, generating metrics");
                let metrics = order_update.order.to_metrics()?;
                for metric in metrics {
                    client.send(metric).await?;
                }
            }
        }
    }
    Ok(())
}
