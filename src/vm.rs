use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::sandbox::Sandbox;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionResult {
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub exit_code: i32,
}

pub struct VirtualMachine {
    pub sandbox: Sandbox,
    pub tool_handlers: HashMap<String, Box<dyn Fn(&serde_json::Value, &Sandbox) -> ExecutionResult>>,
}

impl VirtualMachine {
    pub fn new(sandbox: Sandbox) -> Self {
        let mut vm = Self {
            sandbox,
            tool_handlers: HashMap::new(),
        };
        vm.register_defaults();
        vm
    }

    pub fn register<F>(&mut self, target: &str, action: &str, handler: F)
    where
        F: Fn(&serde_json::Value, &Sandbox) -> ExecutionResult + 'static,
    {
        let key = format!("{}::{}", target, action);
        self.tool_handlers.insert(key, Box::new(handler));
    }

    pub fn dispatch(&self, target: &str, action: &str, payload: &serde_json::Value) -> ExecutionResult {
        let key = format!("{}::{}", target, action);
        if let Some(handler) = self.tool_handlers.get(&key) {
            handler(payload, &self.sandbox)
        } else {
            ExecutionResult {
                ok: false,
                data: None,
                error: Some(format!("Unknown tool contract: {}", key)),
                exit_code: 1,
            }
        }
    }

    fn register_defaults(&mut self) {
        self.register("system", "clock", |_payload, _sandbox| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            ExecutionResult {
                ok: true,
                data: Some(serde_json::json!({ "epoch_ms": now })),
                error: None,
                exit_code: 0,
            }
        });

        self.register("sqlite", "query", |payload, sandbox| {
            let db = match payload.get("db").and_then(|v| v.as_str()) {
                Some(d) => d,
                None => return ExecutionResult { ok: false, data: None, error: Some("Missing db parameter".into()), exit_code: 1 },
            };
            if db.contains("..") {
                return ExecutionResult {
                    ok: false,
                    data: None,
                    error: Some("Security violation: Path escapes sandbox".into()),
                    exit_code: 1,
                };
            }
            // Mock query success for valid path inside sandbox
            match sandbox.validate_path(db) {
                Ok(_) => ExecutionResult {
                    ok: true,
                    data: Some(serde_json::json!({ "rows": [] })),
                    error: None,
                    exit_code: 0,
                },
                Err(e) => ExecutionResult {
                    ok: false,
                    data: None,
                    error: Some(e),
                    exit_code: 1,
                },
            }
        });
    }

    pub fn eval_pir_script(&self, source: &str) -> ExecutionResult {
        // Basic syntax/opcode check for testbed
        if source.contains("INVALID_OPCODE") {
            return ExecutionResult {
                ok: false,
                data: None,
                error: Some("VM TRAP: Invalid opcode encountered".into()),
                exit_code: 1,
            };
        }
        ExecutionResult {
            ok: true,
            data: Some(serde_json::json!({ "status": "success" })),
            error: None,
            exit_code: 0,
        }
    }
}
