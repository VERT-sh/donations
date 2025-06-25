use std::net::{Ipv4Addr, SocketAddrV4};

use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

mod config;
mod routes;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::init();

    let global_filter = EnvFilter::new("service=debug");
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(global_filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("failed to set global tracing subscriber");

    tracing::info!("starting service...");

    let app = routes::router();
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3000)).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
