use std::sync::Arc;

use axum::{self, Router, extract::State, response::IntoResponse, routing::get};
use tower_http::trace::{self, TraceLayer};
use tracing::{Level, info};

use crate::{
    models::{Highway, HighwayNode},
    svg,
};

async fn get_svg(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let highway = state
        .highways
        .iter()
        .find(|way| way.id == 23053042)
        .unwrap();
    info!("road: {:?}", highway);

    let mut connecting: Vec<_> = state
        .highways
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
                .map(|id| state.highways_nodes.iter().find(|n| n.id == *id).unwrap())
                .collect()
        })
        .collect();

    svg::draw_nodes(ordered_highways_nodes)
}

#[derive(Debug)]
pub struct AppState {
    pub highways: Vec<Highway>,
    pub highways_nodes: Vec<HighwayNode>,
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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
