use std::{
    net::SocketAddr,
    sync::{mpsc::Receiver, OnceLock, RwLock},
    thread,
};

use axum::{
    extract::{
        ws::{self, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::twitch::{pubsub::send_to_error_log, ChannelMessages};

static SENDERS: OnceLock<RwLock<Vec<UnboundedSender<ChannelMessages>>>> = OnceLock::new();

async fn websocket(
    websocket_upgrade: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    dbg!("websocket handler function");
    send_to_error_log("websockets: trying to set up websocket".to_string(), "".to_string());
    websocket_upgrade.on_upgrade(move |socket| handle_socket(socket, address))
    // websocket_upgrade.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket, _socket_address: SocketAddr) {
    // async fn handle_socket(mut socket: WebSocket) {
    let (sender, mut receiver) = unbounded_channel::<ChannelMessages>();

    {
        let senders = SENDERS.get_or_init(|| RwLock::new(Vec::new()));
        let mut senders_write = senders.write().unwrap();
        senders_write.push(sender.clone());
        drop(senders_write);
    }

    // NOTE: This loop somehow prevents more than one websocket client from
    // connecting to the server, don't know why yet.
    loop {
        let new_message = receiver.try_recv();
        if let Ok(message) = new_message {
            let json = serde_json::to_string(&message).unwrap();
            if (socket.send(ws::Message::Text(json)).await).is_err() {
                break;
            }
        }
    }

    dbg!("websockets: done setting up websocket".to_string());
}

pub async fn start_websocket(messages_rx: Receiver<ChannelMessages>) {
    thread::spawn(move || loop {
        if let Ok(new_message) = messages_rx.try_recv() {
            let senders = SENDERS.get_or_init(|| RwLock::new(Vec::new()));
            let senders_read = senders.read().unwrap();

            for sender in senders_read.iter() {
                let _ = sender.send(new_message.clone());
            }
        }
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "websockets=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let router = Router::new()
        .route("/", get(websocket))
        .route("/1", get(websocket))
        .route("/2", get(websocket))
        .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:54321").await;
    if listener.is_err() {
        dbg!("Error binding to address");
        return;
    }
    let listener = listener.unwrap();
    dbg!("Local address is: {}", listener.local_addr().unwrap());

    axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
    // axum::serve(listener, router.into_make_service()).await.unwrap();
}
