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

    let filename = "osm-files/midi-pyrenees-latest.osm.pbf";
    let highways = cache::highway_cached(filename)?;
    let highways_nodes = {
        let highways_nodes_ids: Vec<_> = highways.iter().flat_map(|h| h.nodes_id.clone()).collect();
        cache::nodes_cached(filename, highways_nodes_ids)?
    };

    let highway = highways.iter().find(|way| way.id == 23053042).unwrap();
    info!("road: {:?}", highway);

    let mut connecting: Vec<_> = highways
        .iter()
        .filter(|h| h.nodes_id.iter().any(|n| highway.nodes_id.contains(n)))
        .collect();
    info!("Connecting highway: {}", connecting.len());

    connecting.push(highway);

    let ordered_highways_nodes: Vec<Vec<_>> = connecting
        .into_iter()
        .map(|highway| {
            highway
                .nodes_id
                .iter()
                .map(|id| highways_nodes.iter().find(|n| n.id == *id).unwrap())
                .collect()
        })
        .collect();

    debug::draw_svg(ordered_highways_nodes);

    Ok(())
}
