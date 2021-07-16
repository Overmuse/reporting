use crate::REGISTRY;
use std::net::{Ipv4Addr, SocketAddrV4};
use tracing::warn;
use warp::reply::Reply;
use warp::{get, path, serve, Filter, Rejection};

async fn metrics_handler() -> Result<impl Reply, Rejection> {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        warn!("could not encode custom metrics: {}", e);
    };
    let mut res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            warn!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        warn!("could not encode prometheus metrics: {}", e);
    };
    let res_custom = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            warn!("prometheus metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    res.push_str(&res_custom);
    Ok(res)
}

#[tracing::instrument]
pub async fn run(port: u16) {
    let health = path!("health").map(|| "");
    let metrics = path("metrics").and_then(metrics_handler);
    let routes = get().and(health).or(metrics);
    let address = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
    serve(routes).run(address).await
}
