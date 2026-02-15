use axum::{
    extract::{Json, State, Path as AxumPath},
    response::{IntoResponse, Sse, sse::Event},
    routing::{get, post},
    Router,
};
use engine::OmniMind;
use log::info;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use futures_util::stream::{self, Stream};
use tokio_stream::StreamExt;

#[derive(Clone)]
struct AppState {
    mind: Arc<Mutex<OmniMind>>,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Omni Forge API v1...");

    let mind = OmniMind::new_forge("./api_data");
    let state = AppState {
        mind: Arc::new(Mutex::new(mind)),
    };

    let v1_routes = Router::new()
        .route("/learn", post(learn))
        .route("/ask", post(ask))
        .route("/query/stream", get(query_stream))
        .route("/ingest", post(ingest_file))
        .route("/snapshot/:name", post(save_snapshot))
        .route("/snapshot/:name", get(load_snapshot))
        .with_state(state.clone());

    let app = Router::new()
        .route("/", get(root))
        .nest("/v1", v1_routes);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Omni Forge API Industrial Core v1.0"
}

#[derive(Deserialize)]
struct TextPayload {
    text: String,
}

async fn learn(
    State(state): State<AppState>,
    Json(payload): Json<TextPayload>,
) -> impl IntoResponse {
    let mut mind = state.mind.lock().unwrap();
    mind.learn(&payload.text);
    "Learned"
}

#[derive(Serialize)]
struct AskResponse {
    answer: String,
}

async fn ask(State(state): State<AppState>, Json(payload): Json<TextPayload>) -> Json<AskResponse> {
    let mut mind = state.mind.lock().unwrap();
    let answer = mind.ask(&payload.text);
    Json(AskResponse { answer })
}

#[derive(Deserialize)]
struct IngestPayload {
    path: String,
}

async fn ingest_file(
    State(state): State<AppState>,
    Json(payload): Json<IngestPayload>,
) -> impl IntoResponse {
    let mut mind = state.mind.lock().unwrap();
    match mind.ingest_file(&payload.path) {
        Ok(_) => "Ingested",
        Err(e) => {
            info!("Ingestion error: {}", e);
            "Error ingesting"
        }
    }
}

async fn save_snapshot(
    State(state): State<AppState>,
    AxumPath(name): AxumPath<String>,
) -> impl IntoResponse {
    let mind = state.mind.lock().unwrap();
    match mind.save(&name) {
        Ok(_) => "Snapshot Saved",
        Err(_) => "Error saving snapshot",
    }
}

async fn load_snapshot(
    State(state): State<AppState>,
    AxumPath(name): AxumPath<String>,
) -> impl IntoResponse {
    let mut mind = state.mind.lock().unwrap();
    match mind.load(&name) {
        Ok(_) => "Snapshot Loaded",
        Err(_) => "Error loading snapshot",
    }
}

async fn query_stream(
    State(_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    info!("Starting reasoning stream...");

    let stream = stream::repeat_with(|| {
        Event::default().data("Processing reasoning step...")
    })
    .take(5)
    .chain(stream::once(async {
        Event::default().data("Finalizing output.")
    }))
    .map(Ok);

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
