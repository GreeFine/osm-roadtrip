use axum::{
    self, Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use geo::Line;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{Level, info};

use crate::{models::Highway, svg};

fn get_connecing<'b>(
    highways_to_lookup: &Vec<&Highway>,
    mut nearby_highways: Vec<&'b Highway>,
    depth: u64,
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
    road_id: Option<i64>,
    depth: Option<u64>,
    bbox: Option<f64>,
    lon: f64,
    lat: f64,
}

async fn get_svg(
    State(state): State<Arc<AppState>>,
    Query(param): Query<QueryParam>,
) -> impl IntoResponse {
    let connecting = nodes_close(&state, param);

    svg::draw_nodes(connecting)
}

#[derive(Debug, Serialize)]
struct Pathing {
    color: &'static str,
    path: Vec<[f64; 2]>,
}

async fn get_nodes(
    State(state): State<Arc<AppState>>,
    Query(param): Query<QueryParam>,
) -> impl IntoResponse {
    let connecting = nodes_close(&state, param);
    let nodes_geo_array: Vec<Pathing> = connecting
        .into_iter()
        .map(|h| Pathing {
            color: "startNodeFill",
            path: h.nodes.iter().map(|n| [n.longitude, n.latitude]).collect(),
        })
        .collect();
    Json(nodes_geo_array)
}

fn nodes_close(state: &Arc<AppState>, param: QueryParam) -> Vec<&Highway> {
    let highway = if let Some(road_id) = param.road_id {
        state.highways.iter().find(|way| way.id == road_id).unwrap()
    } else {
        state
            .highways
            .iter()
            .find(|way| {
                way.nodes.iter().any(|n| {
                    (n.longitude - param.lon).abs() < 0.001
                        && (n.latitude - param.lat).abs() < 0.001
                })
            })
            .unwrap()
    };
    info!("road: {:?}", highway);

    info!("Getting connections");
    let root_coord = highway.nodes.first().unwrap().coord_epsg3857();
    let close_enough_highways: Vec<&_> = state
        .highways
        .iter()
        .filter(|h| {
            let first_coord = h.nodes.first().unwrap().coord_epsg3857();
            let delta = Line::new(first_coord, root_coord).delta();
            delta.x.abs() < param.bbox.unwrap_or(10_000.)
                && delta.y.abs() < param.bbox.unwrap_or(10_000.)
        })
        .collect();
    let mut connecting = get_connecing(
        &vec![&highway],
        close_enough_highways,
        param.depth.unwrap_or(15),
    );
    info!("Connecting highway: {}", connecting.len());
    connecting.push(highway);
    connecting
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
        .route("/nodes", get(get_nodes))
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
