use alpaca::AlpacaMessage;
use anyhow::Result;
use dogstatsd::Client;
use futures::StreamExt;
use kafka_settings::consumer;
use rdkafka::Message;
mod settings;
mod types;
use types::ToMetrics;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = settings::Settings::new()?;
    let consumer = consumer(&settings.kafka)?;
    let client = Client::new("0.0.0.0:0", &settings.app.target_address).await?;
    while let Some(message) = consumer.stream().next().await {
        let message = message?;
        if let Some(payload) = message.payload() {
            let alpaca_message: AlpacaMessage = serde_json::from_slice(payload)?;
            if let AlpacaMessage::TradeUpdates(order_update) = alpaca_message {
                let metrics = order_update.order.to_metrics()?;
                for metric in metrics {
                    client.send(metric).await?;
                }
            }
        }
    }
    Ok(())
}
