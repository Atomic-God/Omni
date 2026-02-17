#![deny(warnings)]

mod error;
mod config;

use axum::{
    extract::{Json, State, Path as AxumPath},
    response::{Sse, sse::Event},
    routing::{get, post},
    Router,
};
use engine::OmniMind;
use engine::learning::LearningEngine;
use tracing::{info, warn, level_filters::LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use futures_util::stream::{self, Stream};
use tokio_stream::StreamExt;
use error::ApiError;
use config::AppConfig;

#[derive(Clone)]
struct AppState {
    mind: Arc<Mutex<OmniMind>>,
    config: AppConfig,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load Config
    let cfg = AppConfig::load()?;
    cfg.validate().map_err(|e| anyhow::anyhow!(e))?;

    // 2. Initialize Tracing
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    info!("Starting Omni Forge API v1 Industrial Core...");

    let mind = OmniMind::new_forge(cfg.data_dir.to_str().unwrap_or("./api_data"));
    let state = AppState {
        mind: Arc::new(Mutex::new(mind)),
        config: cfg.clone(),
    };

    let v1_routes = Router::new()
        .route("/learn", post(learn))
        .route("/ask", post(ask))
        .route("/query/stream", get(query_stream))
        .route("/feedback", post(feedback))
        .route("/ingest", post(ingest_file))
        .route("/snapshot/:name", post(save_snapshot))
        .route("/snapshot/:name", get(load_snapshot))
        .with_state(state.clone());

    let app = Router::new()
        .route("/", get(root))
        .nest("/v1", v1_routes);

    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.port));
    info!("Industrial Intelligence Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
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
) -> Result<&'static str, ApiError> {
    let mut mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    mind.learn(&payload.text);
    Ok("Learned with Meaning Extraction")
}

async fn ask(
    State(state): State<AppState>,
    Json(payload): Json<TextPayload>
) -> Result<Json<AskResponse>, ApiError> {
    let mut mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    let response = mind.ask(&payload.text);
    Ok(Json(AskResponse {
        answer: response.answer,
        trace: response.trace,
    }))
}

#[derive(Serialize)]
struct AskResponse {
    answer: String,
    trace: Option<engine::ReasoningTrace>,
}

async fn feedback(
    State(state): State<AppState>,
    Json(payload): Json<FeedbackPayload>,
) -> Result<&'static str, ApiError> {
    let mut mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    LearningEngine::process_feedback(&mut mind, &payload.key, payload.score);
    Ok("Feedback Processed")
}

#[derive(Deserialize)]
struct IngestPayload {
    path: String,
}

async fn ingest_file(
    State(state): State<AppState>,
    Json(payload): Json<IngestPayload>,
) -> Result<&'static str, ApiError> {
    let mut mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    match mind.ingest_file(&payload.path) {
        Ok(_) => Ok("Ingested & Extracted Meaning"),
        Err(e) => {
            warn!("Ingestion error: {}", e);
            Err(ApiError::BadRequest(e.to_string()))
        }
    }
}

async fn save_snapshot(
    State(state): State<AppState>,
    AxumPath(name): AxumPath<String>,
) -> Result<&'static str, ApiError> {
    let mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    match mind.save(&name) {
        Ok(_) => Ok("Snapshot Saved"),
        Err(e) => Err(ApiError::Internal(e.to_string())),
    }
}

async fn load_snapshot(
    State(state): State<AppState>,
    AxumPath(name): AxumPath<String>,
) -> Result<&'static str, ApiError> {
    let mut mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    match mind.load(&name) {
        Ok(_) => Ok("Snapshot Loaded"),
        Err(e) => Err(ApiError::Internal(e.to_string())),
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
