mod api;
mod cache;
mod models;
mod projection;
mod svg;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let filename = "osm-files/midi-pyrenees-latest.osm.pbf";
    let highways = cache::highways(filename)?;
    let state = api::AppState { highways };

    api::run(state).await;

    Ok(())
}
