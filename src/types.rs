use alpaca::Order;
use anyhow::{Context, Result};
use dogstatsd::Metric;

pub trait ToMetrics {
    fn to_metrics(&self) -> Result<Vec<Metric>>;
}

impl ToMetrics for Order {
    fn to_metrics(&self) -> Result<Vec<Metric>> {
        let metrics = vec![Metric::increase("order")
            .add_key_value(
                "status",
                serde_plain::to_string(&self.status).context("Failed to serialize order status")?,
            )
            .add_key_value("ticker", &self.symbol)
            .add_key_value(
                "side",
                serde_plain::to_string(&self.side).context("Failed to serialize order side")?,
            )];
        Ok(metrics)
    }
}
