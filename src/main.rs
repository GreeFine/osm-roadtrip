#![warn(unused_crate_dependencies)]
mod api;
mod cache;
mod models;
mod projection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    api::run().await;

    Ok(())
}
