use crate::REGISTRY;
use alpaca::{AlpacaMessage, Event};
use lazy_static::lazy_static;
use prometheus::{CounterVec, GaugeVec, IntCounterVec, Opts};
use rust_decimal::prelude::*;
use tracing::debug;

lazy_static! {
    pub static ref NUM_TRADES: IntCounterVec =
        IntCounterVec::new(Opts::new("num_trades", "Number of trades"), &["ticker"])
            .expect("Metric can be created");
    pub static ref GROSS_TRADE_AMOUNT: CounterVec = CounterVec::new(
        Opts::new("gross_trade_amount", "Gross dollar amount traded"),
        &["ticker"]
    )
    .expect("Metric can be created");
    pub static ref NET_INVESTMENT_AMOUNT: GaugeVec = GaugeVec::new(
        Opts::new("net_investment_amount", "Net dollar amount allocated"),
        &["strategy", "ticker"]
    )
    .expect("Metric can be created");
}

pub fn register() {
    REGISTRY
        .register(Box::new(NUM_TRADES.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(GROSS_TRADE_AMOUNT.clone()))
        .expect("collector can be registered");
    REGISTRY
        .register(Box::new(NET_INVESTMENT_AMOUNT.clone()))
        .expect("collector can be registered");
}

pub fn handle_alpaca_message(message: AlpacaMessage) {
    if let AlpacaMessage::TradeUpdates(order_update) = message {
        if let Event::Fill { price, .. } = order_update.event {
            debug!("Received order fill update, generating metrics");
            let ticker = order_update.order.symbol;
            let qty = order_update.order.qty.unwrap();
            NUM_TRADES.with_label_values(&[&ticker]).inc();
            GROSS_TRADE_AMOUNT
                .with_label_values(&[&ticker])
                .inc_by((qty * price).abs().to_f64().unwrap());
        }
    }
}
