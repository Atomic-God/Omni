use axum::{
    extract::{Json, State, Path as AxumPath},
    response::{IntoResponse, Sse, sse::Event},
    routing::{get, post},
    Router,
};
use engine::OmniMind;
use engine::learning::LearningEngine;
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
    info!("Starting Omni Forge API v1 Industrial [Full Meaning Extraction]...");

    let mind = OmniMind::new_forge("./api_data");
    let state = AppState {
        mind: Arc::new(Mutex::new(mind)),
    };

    let v1_routes = Router::new()
        .route("/learn", post(learn))
        .route("/ask", post(ask))
        .route("/query/stream", get(query_stream))
        .route("/feedback", post(feedback)) // New
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
    "Omni Forge API Industrial Core v1.0 [Semantic Meaning Enabled]"
}

#[derive(Deserialize)]
struct TextPayload {
    text: String,
}

#[derive(Deserialize)]
struct FeedbackPayload {
    key: String,
    score: f32, // -1.0 to 1.0
}

async fn learn(
    State(state): State<AppState>,
    Json(payload): Json<TextPayload>,
) -> impl IntoResponse {
    let mut mind = state.mind.lock().unwrap();
    mind.learn(&payload.text);
    "Learned with Meaning Extraction"
}

async fn ask(State(state): State<AppState>, Json(payload): Json<TextPayload>) -> Json<AskResponse> {
    let mut mind = state.mind.lock().unwrap();
    let answer = mind.ask(&payload.text);
    Json(AskResponse { answer })
}

#[derive(Serialize)]
struct AskResponse {
    answer: String,
}

async fn feedback(
    State(state): State<AppState>,
    Json(payload): Json<FeedbackPayload>,
) -> impl IntoResponse {
    let mut mind = state.mind.lock().unwrap();
    LearningEngine::process_feedback(&mut mind, &payload.key, payload.score);
    "Feedback Processed"
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
        Ok(_) => "Ingested & Extracted Meaning",
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
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    info!("Starting reasoning stream...");

    let mind = state.mind.lock().unwrap();
    let stats = mind.memory_stats();

    let trace_steps = vec![
        format!("Telemetry: {}", stats),
        "Observe: Scanning semantic neighborhoods...".to_string(),
        "Orient: Resolving causal link weights...".to_string(),
        "Decide: Generating abductive hypotheses...".to_string(),
        "Act: Finalizing response with industrial confidence.".to_string(),
    ];

    let stream = stream::iter(trace_steps)
        .map(|step| {
            Event::default().data(step)
        })
        .map(Ok);

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
