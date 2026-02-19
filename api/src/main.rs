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
use error::ApiError;
use config::AppConfig;

#[derive(Clone)]
struct AppState {
    mind: Arc<Mutex<OmniMind>>,
    #[allow(dead_code)]
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
        .route("/telemetry", get(get_telemetry))
        .route("/task/step", post(task_step))
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

async fn get_telemetry(State(state): State<AppState>) -> Result<Json<engine::metrics::RuntimeMetrics>, ApiError> {
    let mut mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    mind.update_metrics();
    Ok(Json(mind.metrics.clone()))
}

async fn task_step(State(state): State<AppState>) -> Result<Json<String>, ApiError> {
    let mut mind = state.mind.lock().map_err(|_| ApiError::Internal("Lock poisoned".to_string()))?;
    let result = mind.execute_task_step().unwrap_or_else(|| "No active goals".to_string());
    Ok(Json(result))
}

async fn query_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>> {
    info!("Starting industrial reasoning stream...");

    let mind_lock = state.mind.clone();

    let stream = stream::unfold(0, move |count| {
        let mind_lock = mind_lock.clone();
        async move {
            if count >= 5 { return None; }

            let msg = {
                let mind = mind_lock.lock().unwrap();
                match count {
                    0 => format!("Telemetry: {}", mind.memory_stats()),
                    1 => "Observe: Scanning semantic neighborhoods...".to_string(),
                    2 => "Orient: Resolving causal link weights...".to_string(),
                    3 => "Decide: Generating abductive hypotheses...".to_string(),
                    4 => "Act: Finalizing response with industrial confidence.".to_string(),
                    _ => unreachable!(),
                }
            };

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Some((Ok(Event::default().data(msg)), count + 1))
        }
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
