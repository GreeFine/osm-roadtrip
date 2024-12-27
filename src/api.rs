use std::{collections::HashMap, sync::Arc};

use axum::{self, Router, extract::State, response::IntoResponse, routing::get};
use geo::Line;
use osmio::ObjId;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{Level, info};

use crate::{models::Highway, svg};

fn get_connecing<'b>(
    highways_to_lookup: &Vec<&Highway>,
    all_highways: &'b Vec<&Highway>,
    depth: u64,
) -> Vec<&'b Highway> {
    info!(
        "Connection computing depth : {depth}, highways_to_lookup: {}",
        highways_to_lookup.len()
    );
    let mut connecting: Vec<&Highway> = all_highways
        .par_iter()
        .filter_map(|highway| {
            if highway.nodes.iter().any(|highway_node| {
                highways_to_lookup
                    .iter()
                    .any(|hl| hl.nodes.contains(highway_node))
            }) {
                // needed for deref
                return Some(*highway);
            }
            None
        })
        .collect();
    if depth == 0 {
        return connecting;
    }
    let mut depth = get_connecing(&connecting, all_highways, depth - 1);
    depth.append(&mut connecting);
    depth
}

async fn get_svg(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let highway = state
        .highways
        .iter()
        .find(|way| way.id == 23053042)
        .unwrap();
    info!("road: {:?}", highway);

    info!("Getting connections");
    let root_coord = highway.nodes.first().unwrap().coord();
    let close_enough_highways: Vec<&_> = state
        .highways
        .iter()
        .filter(|h| {
            let first_coord = h.nodes.first().unwrap().coord();
            let delta = Line::new(first_coord, root_coord).delta();
            delta.x.abs() < 10_000.0 && delta.y.abs() < 10_000.0
        })
        .collect();
    let mut connecting = get_connecing(&vec![&highway], &close_enough_highways, 30);
    info!("Connecting highway: {}", connecting.len());
    connecting.push(highway);

    svg::draw_nodes(connecting)
}

#[derive(Debug)]
pub struct AppState {
    pub highways: Vec<Highway>,
    pub highway_connections: HashMap<ObjId, Vec<ObjId>>,
}

pub async fn run(state: AppState) {
    let state = Arc::new(state);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Ok" }))
        .route("/svg", get(get_svg))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
