use pseudo_harness::engine_client::{EngineClient, EngineError, EngineRequest};

#[test]
fn test_engine_token_budget_enforcement() {
    let client = EngineClient::default_client(); // max_token_ceiling = 1200

    // Small prompt should pass budget validation
    let small_prompt = "f add(a:n, b:n) [ r (+ a b) ]";
    assert!(client.validate_budget(small_prompt, 100).is_ok());

    // Prompt + max_tokens exceeding 1200 should fail with TokenBudgetExceeded
    let large_prompt = "a".repeat(5000); // 5000 / 4 = 1250 tokens > 1200
    let res = client.validate_budget(&large_prompt, 100);
    assert!(matches!(res, Err(EngineError::TokenBudgetExceeded { .. })));
}

#[test]
fn test_engine_request_formatting() {
    let client = EngineClient::new("psudo-engine-v3", 1200, "http://localhost:8080/v1/generate");
    let prompt = "f process(x:n) [ r x ]";
    let system_prompt = Some("You are a pseudo-lang transpiler.");

    let req = client.format_request(prompt, system_prompt, 200).unwrap();
    assert_eq!(req.model, "psudo-engine-v3");
    assert_eq!(req.prompt, prompt);
    assert_eq!(req.system_prompt, system_prompt.map(String::from));
    assert_eq!(req.max_tokens, 200);

    // Serialization check
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("psudo-engine-v3"));
    assert!(json.contains("process"));
}

#[tokio::test]
async fn test_engine_streaming_generation() {
    let client = EngineClient::default_client();
    let prompt = "f compute(n:n) [ r n ]";

    let mut rx = client.generate_stream(prompt, None, 100).await.unwrap();
    let mut chunks = Vec::new();
    while let Some(chunk_res) = rx.recv().await {
        let chunk = chunk_res.unwrap();
        chunks.push(chunk);
    }

    assert!(!chunks.is_empty());
    let full_text = chunks.concat();
    assert!(full_text.contains("fn generated_function()"));
}

#[tokio::test]
async fn test_engine_non_streaming_generation() {
    let client = EngineClient::default_client();
    let prompt = "f test(x:n) [ r x ]";

    let response = client.generate(prompt, Some("System instructions"), 150).await.unwrap();
    assert!(response.contains("fn generated_function()"));
}
