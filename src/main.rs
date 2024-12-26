use std::collections::HashMap;

mod api;
mod cache;
mod mercator;
mod models;
mod svg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let filename = "osm-files/midi-pyrenees-latest.osm.pbf";
    let highways = cache::highways(filename)?;
    let highway_connections = cache::highway_connections(&highways);
    let state = api::AppState {
        highways,
        highway_connections,
    };

    api::run(state).await;

    Ok(())
}
