use axum::{
    self, Router,
    extract::{
        ConnectInfo, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::{any, get},
};
use axum_extra::{TypedHeader, headers};
use geo::Line;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::LazyLock};
use tokio::sync::mpsc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{self, TraceLayer},
};
use tracing::{Level, info};

use crate::{cache, models::Highway};

async fn get_connecing<'b>(
    highways_to_lookup: &Vec<&Highway>,
    mut nearby_highways: Vec<&'b Highway>,
    depth: u64,
    progress: mpsc::Sender<Vec<&'b Highway>>,
) {
    info!(
        "Connection computing depth : {depth}, highways_to_lookup: {}",
        highways_to_lookup.len()
    );
    let connecting: Vec<&Highway> = nearby_highways
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
    progress.send(connecting.clone()).await.unwrap();
    if depth == 0 {
        return;
    }
    nearby_highways.retain(|hl| !connecting.iter().any(|hc| hc.id == hl.id));
    Box::pin(get_connecing(
        &connecting,
        nearby_highways,
        depth - 1,
        progress,
    ))
    .await;
}

#[derive(Debug, Deserialize)]
struct QueryParam {
    road_id: Option<i64>,
    depth: Option<u64>,
    bbox: Option<f64>,
    lon: f64,
    lat: f64,
}

#[derive(Debug, Serialize)]
struct Pathing {
    color: &'static str,
    path: Vec<[f64; 2]>,
}

async fn nodes_close(param: QueryParam, progress: mpsc::Sender<Vec<&Highway>>) {
    let highway = if let Some(road_id) = param.road_id {
        HIGHWAYS.iter().find(|way| way.id == road_id).unwrap()
    } else {
        let Some(highway) = HIGHWAYS.iter().find(|way| {
            way.nodes.iter().any(|n| {
                (n.longitude - param.lon).abs() < 0.002 && (n.latitude - param.lat).abs() < 0.002
            })
        }) else {
            progress.closed().await;
            return;
        };
        highway
    };
    info!("road: {:?}", highway);
    progress.send(vec![&highway]).await.unwrap();

    info!("Getting connections");
    let root_coord = highway.nodes.first().unwrap().coord_epsg3857();
    let close_enough_highways: Vec<&_> = HIGHWAYS
        .iter()
        .filter(|h| {
            let first_coord = h.nodes.first().unwrap().coord_epsg3857();
            let delta = Line::new(first_coord, root_coord).delta();
            delta.x.abs() < param.bbox.unwrap_or(10_000.)
                && delta.y.abs() < param.bbox.unwrap_or(10_000.)
        })
        .collect();
    get_connecing(
        &vec![&highway],
        close_enough_highways,
        param.depth.unwrap_or(15),
        progress,
    )
    .await;
}

pub static HIGHWAYS: LazyLock<Vec<Highway>> = LazyLock::new(|| {
    let filename = "osm-files/belgium-latest.osm.pbf";
    cache::highways(filename).unwrap()
});

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    tracing::info!("{user_agent} on {addr} connected.");

    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Actual websocket statemachine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, _who: SocketAddr) {
    'socket: loop {
        let command = socket.recv().await.unwrap().unwrap();
        tracing::info!("Received command: {:?}", command);
        match command {
            Message::Text(text) => {
                let param: QueryParam = serde_json::from_str(&text).unwrap();
                // create channel to send progress updates
                let (progress, mut progress_rx) = tokio::sync::mpsc::channel(3);
                tokio::spawn(async move {
                    nodes_close(param, progress).await;
                });

                while let Some(progress) = progress_rx.recv().await {
                    let nodes_geo_array: Vec<Pathing> = progress
                        .into_iter()
                        .map(|h| Pathing {
                            color: "startNodeFill",
                            path: h.nodes.iter().map(|n| [n.longitude, n.latitude]).collect(),
                        })
                        .collect();
                    socket
                        .send(axum::extract::ws::Message::Text(
                            serde_json::to_string(&nodes_geo_array).unwrap().into(),
                        ))
                        .await
                        .unwrap();
                }
            }
            Message::Close(_) => {
                tracing::info!("Client disconnect");
                break 'socket;
            }
            _ => {
                tracing::info!("Unknown command");
            }
        }
    }
}

pub async fn run() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Ok" }))
        .route("/ws", any(ws_handler))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::DEBUG))
                .on_response(trace::DefaultOnResponse::new().level(Level::DEBUG)),
        )
        .layer(CorsLayer::new().allow_origin(Any));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
