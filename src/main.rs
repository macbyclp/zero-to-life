mod llm;
mod simulation;
mod world_data;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use llm::{AgentBrain, LlmConfig};
use serde::{Deserialize, Serialize};
use simulation::SimulationState;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, Mutex};
use tracing::{info, warn};
use world_data::{WorldDataFetcher, WorldSnapshot};

const INDEX_HTML: &str = include_str!("../static/index.html");

#[derive(Debug, Clone, Serialize)]
struct FullPayload {
    running: bool,
    tick_ms: u64,
    llm_config: LlmConfigPublic,
    world: WorldSnapshot,
    sim: SimulationState,
}

#[derive(Debug, Clone, Serialize)]
struct LlmConfigPublic {
    provider: String,
    model: String,
    base_url: String,
    has_api_key: bool,
}

impl From<&LlmConfig> for LlmConfigPublic {
    fn from(c: &LlmConfig) -> Self {
        Self {
            provider: c.provider.clone(),
            model: c.model.clone(),
            base_url: c.base_url.clone(),
            has_api_key: !c.api_key.trim().is_empty(),
        }
    }
}

struct SharedEngine {
    running: bool,
    tick_ms: u64,
    llm_config: LlmConfig,
    world: WorldSnapshot,
    sim: SimulationState,
}

struct AppState {
    engine: Mutex<SharedEngine>,
    fetcher: WorldDataFetcher,
    brain: AgentBrain,
    tx: broadcast::Sender<String>,
    step_lock: Mutex<()>,
}

impl AppState {
    async fn snapshot_payload(&self) -> FullPayload {
        let eng = self.engine.lock().await;
        FullPayload {
            running: eng.running,
            tick_ms: eng.tick_ms,
            llm_config: LlmConfigPublic::from(&eng.llm_config),
            world: eng.world.clone(),
            sim: eng.sim.clone(),
        }
    }

    async fn broadcast_state(&self) {
        let payload = self.snapshot_payload().await;
        if let Ok(json) = serde_json::to_string(&payload) {
            let _ = self.tx.send(json);
        }
    }

    async fn run_one_tick(&self) {
        let _guard = self.step_lock.lock().await;

        let (sim_copy, world_copy, llm_cfg) = {
            let eng = self.engine.lock().await;
            if !eng.sim.alive {
                return;
            }
            (eng.sim.clone(), eng.world.clone(), eng.llm_config.clone())
        };

        let (decision, provider_used) = self.brain.decide(&llm_cfg, &sim_copy, &world_copy).await;

        {
            let mut eng = self.engine.lock().await;
            let world_ref = eng.world.clone();
            eng.sim.apply_decision(decision, &world_ref, provider_used);
            if !eng.sim.alive {
                eng.running = false;
            }
        }

        self.broadcast_state().await;
    }

    async fn refresh_world(&self) {
        let prev = {
            let eng = self.engine.lock().await;
            eng.world.clone()
        };
        let fresh = self.fetcher.fetch_snapshot(&prev).await;
        {
            let mut eng = self.engine.lock().await;
            eng.world = fresh.clone();
            eng.sim.recalculate_net_worth(&fresh);
        }
        self.broadcast_state().await;
    }
}

#[derive(Deserialize)]
struct ControlRequest {
    command: Option<String>,
    tick_ms: Option<u64>,
}

#[derive(Deserialize)]
struct ConfigUpdateRequest {
    provider: String,
    model: String,
    base_url: String,
    api_key: Option<String>,
}

#[derive(Deserialize)]
struct EventInjectRequest {
    title: String,
    cash_delta: f64,
    mood_delta: f64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "zero_to_life=info,axum=info".into()),
        )
        .init();

    let world = WorldSnapshot::default();
    let sim = SimulationState::new(&world);
    let llm_config = LlmConfig::default();

    let (tx, _) = broadcast::channel::<String>(64);
    let state = Arc::new(AppState {
        engine: Mutex::new(SharedEngine {
            running: false,
            tick_ms: 2600,
            llm_config,
            world,
            sim,
        }),
        fetcher: WorldDataFetcher::new(),
        brain: AgentBrain::new(),
        tx,
        step_lock: Mutex::new(()),
    });

    // Initial read-only real-world data fetch & periodic refresh every 2 minutes
    let state_world = Arc::clone(&state);
    tokio::spawn(async move {
        loop {
            state_world.refresh_world().await;
            tokio::time::sleep(Duration::from_secs(120)).await;
        }
    });

    // Background simulation loop
    let state_sim = Arc::clone(&state);
    tokio::spawn(async move {
        loop {
            let (is_running, tick_ms) = {
                let eng = state_sim.engine.lock().await;
                (eng.running && eng.sim.alive, eng.tick_ms)
            };

            if is_running {
                state_sim.run_one_tick().await;
                tokio::time::sleep(Duration::from_millis(tick_ms.clamp(600, 15_000))).await;
            } else {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
        }
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/api/state", get(get_state))
        .route("/api/control", post(post_control))
        .route("/api/config", post(post_config))
        .route("/api/event", post(post_event))
        .route("/api/refresh-world", post(post_refresh_world))
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3030".to_string());
    let addr = format!("127.0.0.1:{}", port);
    info!("zero-to-life simülasyonu hazır: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap_or_else(|e| {
        warn!("Port {} açılamadı: {}", addr, e);
        std::process::exit(1);
    });

    axum::serve(listener, app).await.unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn get_state(State(state): State<Arc<AppState>>) -> Json<FullPayload> {
    Json(state.snapshot_payload().await)
}

async fn post_control(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ControlRequest>,
) -> Json<FullPayload> {
    if let Some(ms) = req.tick_ms {
        let mut eng = state.engine.lock().await;
        eng.tick_ms = ms.clamp(600, 15_000);
    }

    if let Some(cmd) = req.command {
        match cmd.as_str() {
            "play" => {
                let mut eng = state.engine.lock().await;
                if eng.sim.alive {
                    eng.running = true;
                }
            }
            "pause" => {
                let mut eng = state.engine.lock().await;
                eng.running = false;
            }
            "step" => {
                {
                    let mut eng = state.engine.lock().await;
                    eng.running = false;
                }
                state.run_one_tick().await;
                return Json(state.snapshot_payload().await);
            }
            "reset" => {
                let mut eng = state.engine.lock().await;
                eng.running = false;
                let world_ref = eng.world.clone();
                eng.sim = SimulationState::new(&world_ref);
            }
            _ => {}
        }
    }

    state.broadcast_state().await;
    Json(state.snapshot_payload().await)
}

async fn post_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConfigUpdateRequest>,
) -> Json<FullPayload> {
    {
        let mut eng = state.engine.lock().await;
        eng.llm_config.provider = req.provider;
        eng.llm_config.model = req.model;
        eng.llm_config.base_url = req.base_url;
        if let Some(key) = req.api_key {
            if !key.trim().is_empty() {
                eng.llm_config.api_key = key.trim().to_string();
            }
        }
    }
    state.broadcast_state().await;
    Json(state.snapshot_payload().await)
}

async fn post_event(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EventInjectRequest>,
) -> Json<FullPayload> {
    {
        let mut eng = state.engine.lock().await;
        let world_ref = eng.world.clone();
        eng.sim
            .inject_event(req.title, req.cash_delta, req.mood_delta, &world_ref);
    }
    state.broadcast_state().await;
    Json(state.snapshot_payload().await)
}

async fn post_refresh_world(State(state): State<Arc<AppState>>) -> Json<FullPayload> {
    state.refresh_world().await;
    Json(state.snapshot_payload().await)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    // Send initial snapshot immediately upon connect
    if let Ok(initial) = serde_json::to_string(&state.snapshot_payload().await) {
        let _ = sender.send(Message::Text(initial)).await;
    }

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Drain incoming client pings/messages until disconnect
    while let Some(Ok(_)) = receiver.next().await {}
    send_task.abort();
}
