use std::sync::Arc;

use axum::{
    self, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use geo::Line;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{Level, info};

use crate::{models::Highway, svg};

fn get_connecing<'b>(
    highways_to_lookup: &Vec<&Highway>,
    nearby_highways: &'b mut Vec<&Highway>,
    depth: u32,
) -> Vec<&'b Highway> {
    info!(
        "Connection computing depth : {depth}, highways_to_lookup: {}",
        highways_to_lookup.len()
    );
    let mut connecting: Vec<&Highway> = nearby_highways
        .par_iter()
        .filter_map(|highway| {
            if highway.nodes.iter().any(|highway_node| {
                highways_to_lookup
                    .iter()
                    .any(|hl| hl.id != highway_node.id && hl.nodes.contains(highway_node))
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
    nearby_highways.retain(|hl| !connecting.iter().any(|hc| hc.id == hl.id));
    let mut depth = get_connecing(&connecting, nearby_highways, depth - 1);
    depth.append(&mut connecting);
    depth
}

#[derive(Debug, Deserialize)]
struct QueryParam {
    id: i64,
    depth: Option<u32>,
}

async fn get_svg(
    State(state): State<Arc<AppState>>,
    Query(param): Query<QueryParam>,
) -> impl IntoResponse {
    let highway = state
        .highways
        .iter()
        .find(|way| way.id == param.id)
        .unwrap();
    info!("road: {:?}", highway);

    info!("Getting connections");
    let root_coord = highway.nodes.first().unwrap().coord();
    let mut close_enough_highways: Vec<&_> = state
        .highways
        .iter()
        .filter(|h| {
            let first_coord = h.nodes.first().unwrap().coord();
            let delta = Line::new(first_coord, root_coord).delta();
            delta.x.abs() < 10_000.0 && delta.y.abs() < 10_000.0
        })
        .collect();
    let mut connecting = get_connecing(
        &vec![&highway],
        &mut close_enough_highways,
        param.depth.unwrap_or(15),
    );
    info!("Connecting highway: {}", connecting.len());
    connecting.push(highway);

    svg::draw_nodes(connecting)
}

#[derive(Debug)]
pub struct AppState {
    pub highways: Vec<Highway>,
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
