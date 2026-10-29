use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineRequest {
    pub model: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub max_tokens: usize,
    pub temperature: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    TokenBudgetExceeded { used: usize, limit: usize },
    RequestError(String),
    NetworkError(String),
    SerializationError(String),
}

impl std::error::Error for EngineError {}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::TokenBudgetExceeded { used, limit } => {
                write!(
                    f,
                    "Token budget exceeded: used {} tokens, ceiling is {} tokens",
                    used, limit
                )
            }
            EngineError::RequestError(msg) => write!(f, "Engine request error: {}", msg),
            EngineError::NetworkError(msg) => write!(f, "Engine network error: {}", msg),
            EngineError::SerializationError(msg) => write!(f, "Engine serialization error: {}", msg),
        }
    }
}

pub struct EngineClient {
    pub model: String,
    pub max_token_ceiling: usize,
    pub api_endpoint: String,
}

impl EngineClient {
    pub fn new(
        model: impl Into<String>,
        max_token_ceiling: usize,
        api_endpoint: impl Into<String>,
    ) -> Self {
        Self {
            model: model.into(),
            max_token_ceiling,
            api_endpoint: api_endpoint.into(),
        }
    }

    pub fn default_client() -> Self {
        Self {
            model: "psudo-engine-v3".into(),
            max_token_ceiling: 1200,
            api_endpoint: "http://localhost:8080/v1/generate".into(),
        }
    }

    pub fn estimate_tokens(text: &str) -> usize {
        // Heuristic: ~4 characters per token, matching ContextProjector
        text.len() / 4
    }

    pub fn validate_budget(
        &self,
        prompt: &str,
        requested_max_tokens: usize,
    ) -> Result<usize, EngineError> {
        let prompt_tokens = Self::estimate_tokens(prompt);
        let total_tokens = prompt_tokens + requested_max_tokens;
        if total_tokens > self.max_token_ceiling {
            return Err(EngineError::TokenBudgetExceeded {
                used: total_tokens,
                limit: self.max_token_ceiling,
            });
        }
        Ok(total_tokens)
    }

    pub fn format_request(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: usize,
    ) -> Result<EngineRequest, EngineError> {
        let full_text = match system_prompt {
            Some(sys) => format!("{}\n{}", sys, prompt),
            None => prompt.to_string(),
        };
        self.validate_budget(&full_text, max_tokens)?;

        Ok(EngineRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            system_prompt: system_prompt.map(|s| s.to_string()),
            max_tokens,
            temperature: 0.2,
        })
    }

    pub async fn generate_stream(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: usize,
    ) -> Result<mpsc::Receiver<Result<String, EngineError>>, EngineError> {
        // Format and validate budget first
        let _req = self.format_request(prompt, system_prompt, max_tokens)?;

        let (tx, rx) = mpsc::channel(32);

        let prompt_clone = prompt.to_string();
        tokio::spawn(async move {
            let response_chunks = vec![
                format!(
                    "Transpiling pseudo-code for: {} ...\n",
                    &prompt_clone[..prompt_clone.len().min(20)]
                ),
                "fn generated_function() {\n".into(),
                "    let x = 42;\n".into(),
                "    println!(\"Result: {}\", x);\n".into(),
                "}\n".into(),
            ];

            for chunk in response_chunks {
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }

    pub async fn generate(
        &self,
        prompt: &str,
        system_prompt: Option<&str>,
        max_tokens: usize,
    ) -> Result<String, EngineError> {
        let mut rx = self.generate_stream(prompt, system_prompt, max_tokens).await?;
        let mut full_response = String::new();
        while let Some(chunk_result) = rx.recv().await {
            match chunk_result {
                Ok(chunk) => full_response.push_str(&chunk),
                Err(e) => return Err(e),
            }
        }
        Ok(full_response)
    }
}
