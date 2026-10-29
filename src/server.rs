use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::crdt::SymbolMapCRDT;
use crate::engine_client::EngineClient;
use crate::vm::VirtualMachine;

#[derive(Clone)]
pub struct ServerState {
    pub crdt: Arc<Mutex<SymbolMapCRDT>>,
    pub vm: Arc<Mutex<VirtualMachine>>,
    pub engine: Arc<EngineClient>,
    pub token_count: Arc<Mutex<usize>>,
    pub token_ceiling: usize,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub model: String,
    pub vram: String,
    pub tokens_per_second: u32,
    pub token_count: usize,
    pub token_ceiling: usize,
    pub lamport_tick: u64,
}

#[derive(Serialize)]
pub struct SymbolItem {
    pub name: String,
    pub kind: String,
    pub token_cost: usize,
}

#[derive(Deserialize)]
pub struct SymbolQuery {
    pub q: Option<String>,
}

#[derive(Deserialize)]
pub struct PromptRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub struct PromptResponse {
    pub ok: bool,
    pub reply: String,
    pub token_count: usize,
    pub token_ceiling: usize,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct EvalRequest {
    pub code: String,
}

#[derive(Serialize)]
pub struct EvalResponse {
    pub ok: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub fn create_router(state: ServerState, static_dir: Option<&str>) -> Router {
    let mut api_router = Router::new()
        .route("/status", get(get_status))
        .route("/symbols", get(get_symbols))
        .route("/prompt", post(handle_prompt))
        .route("/eval", post(handle_eval))
        .route("/agenda", get(get_agenda))
        .route("/files", get(get_files))
        .with_state(state.clone());

    let cors = CorsLayer::permissive();

    let mut app = Router::new()
        .nest("/api", api_router)
        .layer(cors);

    if let Some(dir) = static_dir {
        if std::path::Path::new(dir).exists() {
            app = app.fallback_service(ServeDir::new(dir));
        } else {
            app = app.fallback(fallback_html);
        }
    } else {
        app = app.fallback(fallback_html);
    }

    app
}

async fn get_status(State(state): State<ServerState>) -> Json<StatusResponse> {
    let crdt = state.crdt.lock().unwrap();
    let tokens = *state.token_count.lock().unwrap();
    Json(StatusResponse {
        model: "3.2B Ternary (Native 2-Bit)".into(),
        vram: "1.4 / 6.0 GB".into(),
        tokens_per_second: 42,
        token_count: tokens,
        token_ceiling: state.token_ceiling,
        lamport_tick: crdt.current_tick,
    })
}

async fn get_symbols(
    State(_state): State<ServerState>,
    Query(query): Query<SymbolQuery>,
) -> Json<Vec<SymbolItem>> {
    let all_symbols = vec![
        SymbolItem { name: "net/client.pseudo".into(), kind: "file".into(), token_cost: 45 },
        SymbolItem { name: "src/main.pseudo".into(), kind: "file".into(), token_cost: 30 },
        SymbolItem { name: "net::client::fetch".into(), kind: "fn".into(), token_cost: 32 },
        SymbolItem { name: "net::client::Config".into(), kind: "struct".into(), token_cost: 20 },
        SymbolItem { name: "storage::engine::Store".into(), kind: "struct".into(), token_cost: 50 },
        SymbolItem { name: "system::clock".into(), kind: "fn".into(), token_cost: 15 },
        SymbolItem { name: "sqlite::query".into(), kind: "fn".into(), token_cost: 25 },
        SymbolItem { name: "crdt::SymbolMapCRDT".into(), kind: "struct".into(), token_cost: 60 },
        SymbolItem { name: "sandbox::Sandbox".into(), kind: "struct".into(), token_cost: 40 },
    ];

    if let Some(q) = query.q {
        let q_lower = q.to_lowercase();
        let filtered = all_symbols
            .into_iter()
            .filter(|s| s.name.to_lowercase().contains(&q_lower))
            .collect();
        Json(filtered)
    } else {
        Json(all_symbols)
    }
}

async fn handle_prompt(
    State(state): State<ServerState>,
    Json(payload): Json<PromptRequest>,
) -> Json<PromptResponse> {
    let added_tokens = payload.prompt.len() / 4 + 10;
    let mut current_tokens = state.token_count.lock().unwrap();

    if *current_tokens + added_tokens > state.token_ceiling {
        return Json(PromptResponse {
            ok: false,
            reply: String::new(),
            token_count: *current_tokens,
            token_ceiling: state.token_ceiling,
            error: Some("Operation rejected: Exceeds 1,200 token operational ceiling".into()),
        });
    }

    *current_tokens += added_tokens;

    // Simulate psudo-engine caveman response with AST output
    let reply = format!(
        "<talk>\nMe read prompt: \"{}\".\nMe write logic in pIR rock.\n</talk>\n\n<pir>\n+ custom.feature [\n    f run () -> :n [\n        v ok (@x \"system\" \"clock\" {{}})\n        r ok.epoch_ms\n    ]\n]\n</pir>",
        payload.prompt
    );

    Json(PromptResponse {
        ok: true,
        reply,
        token_count: *current_tokens,
        token_ceiling: state.token_ceiling,
        error: None,
    })
}

async fn handle_eval(
    State(state): State<ServerState>,
    Json(payload): Json<EvalRequest>,
) -> Json<EvalResponse> {
    let vm = state.vm.lock().unwrap();
    let res = vm.eval_pir_script(&payload.code);
    Json(EvalResponse {
        ok: res.ok,
        output: res.data,
        error: res.error,
    })
}

async fn get_agenda() -> Json<Vec<serde_json::Value>> {
    Json(vec![
        serde_json::json!({ "id": "epic_vm", "title": "Epic 1: Native VM & CRDT Runtime", "status": "completed" }),
        serde_json::json!({ "id": "epic_web", "title": "Epic 2: Rust Server & Svelte SPA Workbench", "status": "in_progress" }),
        serde_json::json!({ "id": "task_spa", "title": "Task 2.1: Svelte 4-Region Dashboard", "status": "completed" }),
        serde_json::json!({ "id": "task_at", "title": "Task 2.2: Live @ Autocomplete Popover", "status": "completed" }),
        serde_json::json!({ "id": "epic_engine", "title": "Epic 3: psudo-engine VRAM Streaming Bridge", "status": "pending" }),
    ])
}

async fn get_files() -> Json<Vec<String>> {
    Json(vec![
        "src/main.rs".into(),
        "src/lib.rs".into(),
        "src/server.rs".into(),
        "src/crdt.rs".into(),
        "src/sandbox.rs".into(),
        "src/vm.rs".into(),
        "src/engine_client.rs".into(),
        ".pseudo/main.pir".into(),
        ".pseudo/todo.json".into(),
        "Cargo.toml".into(),
        "README.md".into(),
        "SPEC.md".into(),
    ])
}

async fn fallback_html() -> impl IntoResponse {
    let embedded_html = include_str!("../static/index.html");
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8"))
        .body(embedded_html.to_string())
        .unwrap()
}
