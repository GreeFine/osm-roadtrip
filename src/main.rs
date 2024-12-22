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
    let highways = cache::highway_cached(filename)?;
    let highways_nodes = {
        let highways_nodes_ids: Vec<_> = highways.iter().flat_map(|h| h.nodes_id.clone()).collect();
        cache::nodes_cached(filename, highways_nodes_ids)?
    };

    let state = api::AppState {
        highways,
        highways_nodes,
    };

    api::run(state).await;

    Ok(())
}
