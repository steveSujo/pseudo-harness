use std::sync::{Arc, Mutex};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // for oneshot
use pseudo_harness::crdt::SymbolMapCRDT;
use pseudo_harness::engine_client::EngineClient;
use pseudo_harness::sandbox::Sandbox;
use pseudo_harness::server::{create_router, ServerState};
use pseudo_harness::vm::VirtualMachine;
use tempfile::TempDir;

fn setup_test_state() -> ServerState {
    let tmp = TempDir::new().unwrap();
    let sandbox = Sandbox::new(tmp.path()).unwrap();
    ServerState {
        crdt: Arc::new(Mutex::new(SymbolMapCRDT::new(10))),
        vm: Arc::new(Mutex::new(VirtualMachine::new(sandbox))),
        engine: Arc::new(EngineClient::default_client()),
        token_count: Arc::new(Mutex::new(100)),
        token_ceiling: 1200,
    }
}

#[tokio::test]
async fn test_api_status_endpoint() {
    let state = setup_test_state();
    let app = create_router(state, None);

    let req = Request::builder()
        .uri("/api/status")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_symbols_query() {
    let state = setup_test_state();
    let app = create_router(state, None);

    let req = Request::builder()
        .uri("/api/symbols?q=net")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_prompt_and_token_ceiling() {
    let state = setup_test_state();
    let app = create_router(state, None);

    let payload = serde_json::json!({ "prompt": "build http client" }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/prompt")
        .header("content-type", "application/json")
        .body(Body::from(payload))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
