use axum::{
    extract::{Json, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use engine::OmniMind;
use log::info;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    mind: Arc<Mutex<OmniMind>>,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting Omni Forge API...");

    let mind = OmniMind::new();
    // Try load memory
    let state = AppState {
        mind: Arc::new(Mutex::new(mind)),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/learn", post(learn))
        .route("/ask", post(ask))
        .route("/save", post(save))
        .route("/load", post(load))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Omni Forge API v5.0"
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
    let mind = state.mind.lock().unwrap();
    let answer = mind.ask(&payload.text);
    Json(AskResponse { answer })
}

#[derive(Deserialize)]
struct PathPayload {
    path: String,
}

async fn save(
    State(state): State<AppState>,
    Json(payload): Json<PathPayload>,
) -> impl IntoResponse {
    let mind = state.mind.lock().unwrap();
    match mind.save(&payload.path) {
        Ok(_) => "Saved",
        Err(_) => "Error saving",
    }
}

async fn load(
    State(state): State<AppState>,
    Json(payload): Json<PathPayload>,
) -> impl IntoResponse {
    let mut mind = state.mind.lock().unwrap();
    match mind.load(&payload.path) {
        Ok(_) => "Loaded",
        Err(_) => "Error loading",
    }
}
