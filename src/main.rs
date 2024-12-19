use log::info;

mod cache;
mod debug;
mod mercator;
mod models;

fn main() -> anyhow::Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "debug");
    }
    pretty_env_logger::env_logger::init();

    let filename = "midi-pyrenees-latest.osm.pbf";
    let highways = cache::highway_cached(filename)?;
    let highways_nodes_ids: Vec<_> = highways.iter().flat_map(|h| h.nodes.clone()).collect();
    let highways_nodes = cache::nodes_cached(filename, highways_nodes_ids)?;

    let highway = highways.iter().find(|way| way.id == 23053042).unwrap();
    info!("road: {:?}", highway);

    let ordered_highway_nodes: Vec<_> = highway
        .nodes
        .iter()
        .map(|id| highways_nodes.iter().find(|n| n.id == *id).unwrap())
        .collect();

    debug::draw_svg(ordered_highway_nodes);

    Ok(())
}
