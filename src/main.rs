use std::collections::HashSet;
use std::sync::Arc;

use askama_axum::{IntoResponse, Template};
use axum::body::Body;
use axum::extract::Query;
use axum::response::Redirect;
use axum::Router;
use axum::{extract::State, routing::get};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use circular_buffer::CircularBuffer;
use clap::Parser;
use commonlib::auth::Auth;
use rtmp::rtmp::RtmpServer;
use streamhub::StreamsHub;

mod chat;
mod config;
use config::Config;
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;

use crate::chat::chat_ws_handler;

#[derive(Parser, Debug)]
struct CliArgs {
    /// Path to the toml config file
    #[arg(long)]
    config: Option<std::path::PathBuf>,
}

/// State of the app
struct AppState {
    /// config of the server
    config: Config,
    /// channels to send messages internally to all the websocket handlers
    tx: broadcast::Sender<chat::Message>,
    /// users in the chat room
    users: Mutex<HashSet<String>>,
    /// history of the chat, stores the last 25 messages
    _history: Mutex<CircularBuffer<25, chat::Message>>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: Option<String>,
    is_authed: bool,
}

impl IndexTemplate {
    pub async fn handler(State(state): State<Arc<AppState>>, jar: CookieJar) -> IndexTemplate {
        let is_authed = if let Some(key) = state.config.key() {
            let tok = jar.get("auth_tok");
            tok.map(|tok| tok.value() == key).unwrap_or(false)
        } else {
            true
        };

        IndexTemplate {
            title: state.config.title(),
            is_authed,
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = CliArgs::parse();

    let config = if let Some(config_path) = args.config {
        Config::load_from_file(&config_path).unwrap()
    } else {
        tracing::info!("No config provided, this is illegal");
        Config::default()
    };

    // start the rtmp server
    let rtmp_port = config.rtmp_port();
    let mut stream_hub = StreamsHub::new(None);
    let strm_sender = stream_hub.get_hub_event_sender();

    let auth = Auth::new(
        "".to_string(), // this is ignored if auth algo is AuthAlogrithm::Simple
        config.stream_key().to_string(),
        commonlib::auth::AuthAlgorithm::Simple,
        commonlib::auth::AuthType::Push,
    );

    {
        let auth = auth.clone();
        tokio::spawn(async move {
            let mut rtmp_server = RtmpServer::new(
                format!("0.0.0.0:{rtmp_port}"),
                strm_sender,
                1,
                Some(auth.clone()),
            );
            if let Err(e) = rtmp_server.run().await {
                tracing::error!("RtmpServer exited: {e}");
            }
        });
    }

    let strm_sender = stream_hub.get_hub_event_sender();
    let port = config.port();
    tokio::spawn(async move {
        if let Err(e) = httpflv::server::run(strm_sender, (port + 1) as usize, Some(auth)).await {
            tracing::error!("httpflv exited: {e}");
        }
    });

    tokio::spawn(async move {
        stream_hub.run().await;
    });

    let (tx, _rx) = broadcast::channel(100);
    let address = format!("127.0.0.1:{}", config.port());
    let router = Router::new()
        .route("/", get(IndexTemplate::handler))
        .route("/live", get(live_proxy))
        .route("/chat", get(chat_ws_handler))
        .route("/auth", get(auth_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(Arc::new(AppState {
            config,
            tx,
            users: Mutex::new(HashSet::new()),
            _history: Mutex::new(CircularBuffer::new()),
        }));

    let listener = tokio::net::TcpListener::bind(address.to_string())
        .await
        .unwrap();

    tracing::info!("Server running on http://{address}");
    axum::serve(listener, router).await.unwrap();
}

async fn live_proxy(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let uri = format!(
        "http://127.0.0.1:{}/live/x0/movie.flv",
        state.config.port() + 1
    );

    match reqwest::get(uri).await {
        Ok(res) => Body::from_stream(res.bytes_stream()),
        Err(e) => {
            tracing::error!("Failed to proxy to /live/x0/movie.flv: {e}");
            Body::empty()
        }
    }
}

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct AuthParams {
    key: String,
}

async fn auth_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AuthParams>,
) -> (CookieJar, Redirect) {
    tracing::info!("Auth Request: {params:?}");
    let matches = state
        .config
        .key()
        .map(|key| params.key == key)
        .unwrap_or(true);

    let redirector = Redirect::to("/");

    if !matches {
        tracing::info!("Failed Auth Request");
        return (CookieJar::new(), redirector.clone());
    }

    (
        CookieJar::new().add(Cookie::new("auth_tok", params.key)),
        redirector,
    )
}
