use std::{collections::HashMap, sync::Arc};

use axum::{self, Router, extract::State, response::IntoResponse, routing::get};
use osmio::ObjId;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{Level, info};

use crate::{models::Highway, svg};

fn get_connecing<'a, 'b>(
    highways_to_lookup: &'a Vec<&Highway>,
    all_highways: &'b Vec<Highway>,
    depth: u64,
) -> Vec<&'b Highway> {
    info!(
        "Connection computing depth : {depth}, highways_to_lookup: {}",
        highways_to_lookup.len()
    );
    let mut connecting: Vec<_> = all_highways
        .par_iter()
        .filter(|all_highway| {
            all_highway.nodes.iter().any(|all_highway_node| {
                highways_to_lookup
                    .iter()
                    .any(|hl| hl.nodes.contains(all_highway_node))
            })
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
    // let highway = state
    //     .highways
    //     .iter()
    //     .find(|way| way.id == 23053042)
    //     .unwrap();
    // info!("road: {:?}", highway);

    // info!("Getting connections");
    // let mut connecting = get_connecing(&vec![&highway], &state.highways, 5);
    // info!("Connecting highway: {}", connecting.len());
    // connecting.push(highway);
    let to_draw: Vec<_> = state
        .highway_connections
        .values()
        .flat_map(|ids| {
            ids.iter().map(|id| {
                state
                    .highways
                    .get(state.highways.binary_search_by(|h| h.id.cmp(id)).unwrap())
                    .unwrap()
            })
        })
        .collect();
    svg::draw_nodes(to_draw)
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
