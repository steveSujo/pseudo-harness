use crate::crdt::SymbolMapCRDT;

pub struct ContextProjector {
    max_budget: usize,
}

impl ContextProjector {
    pub fn new(max_budget: usize) -> Self {
        Self { max_budget }
    }

    pub fn estimate_tokens(text: &str) -> usize {
        // Simple heuristic: ~4 characters per token
        text.len() / 4
    }

    pub fn build_slice(
        &self,
        crdt: &SymbolMapCRDT,
        target_symbol_id: &str,
        last_error_frame: &str,
    ) -> Result<String, String> {
        let mut slice = String::new();
        let mut used_tokens = 0;

        // 1. Tool contracts (~150 tokens)
        let contracts = Self::get_tool_contracts();
        let c_tokens = Self::estimate_tokens(&contracts);
        slice.push_str(&contracts);
        slice.push('\n');
        used_tokens += c_tokens;

        // 2. Transient error frame if failed
        if !last_error_frame.is_empty() {
            let err_block = format!("<error>\n{}\n</error>\n", last_error_frame);
            let e_tokens = Self::estimate_tokens(&err_block);
            if used_tokens + e_tokens > self.max_budget {
                return Err("Context budget exceeded by error frame".into());
            }
            slice.push_str(&err_block);
            used_tokens += e_tokens;
        }

        // 3. Target symbol & 1-hop dependencies
        if let Some(target_node) = crdt.symbols.get(target_symbol_id) {
            let body_block = format!("// TARGET DEFINITION: {}\n{}\n", target_node.id, target_node.body);
            let b_tokens = Self::estimate_tokens(&body_block);
            if used_tokens + b_tokens > self.max_budget {
                return Err("Context budget exceeded by target definition".into());
            }
            slice.push_str(&body_block);
            used_tokens += b_tokens;

            for dep_id in &target_node.dependencies {
                if let Some(dep_node) = crdt.symbols.get(dep_id) {
                    if !dep_node.is_tombstone {
                        let sig = format!("// DEP: {}\n", dep_id);
                        let s_tokens = Self::estimate_tokens(&sig);
                        if used_tokens + s_tokens <= self.max_budget {
                            slice.push_str(&sig);
                            used_tokens += s_tokens;
                        }
                    }
                }
            }
        }

        if used_tokens > self.max_budget {
            return Err("Context budget exceeded 1,200 token ceiling".into());
        }

        Ok(slice)
    }

    fn get_tool_contracts() -> String {
        "// TOOL SIGNATURES\n@x \"http\" \"get\" {url: :s, timeout_ms: :n} -> {status: :n, body: :s}\n@x \"sqlite\" \"query\" {db: :s, query: :s, params: :l} -> {rows: :l}\n@x \"system\" \"clock\" {} -> {epoch_ms: :n}".into()
    }
}
