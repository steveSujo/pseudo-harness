use std::collections::HashMap;
use xxhash_rust::xxh64::xxh64;

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolNode {
    pub id: String,
    pub lamport: u64,
    pub content_hash: u64,
    pub dependencies: Vec<String>,
    pub body: String,
    pub is_tombstone: bool,
    pub deleted_tick: u64,
}

pub struct SymbolMapCRDT {
    pub symbols: HashMap<String, SymbolNode>,
    pub current_tick: u64,
    pub tombstone_ttl: u64,
}

impl SymbolMapCRDT {
    pub fn new(tombstone_ttl: u64) -> Self {
        Self {
            symbols: HashMap::new(),
            current_tick: 0,
            tombstone_ttl,
        }
    }

    pub fn advance_tick(&mut self) {
        self.current_tick += 1;
        self.prune_tombstones();
    }

    pub fn apply_patch(
        &mut self,
        id: String,
        lamport: u64,
        content_hash: u64,
        dependencies: Vec<String>,
        body: String,
        is_delete: bool,
    ) -> bool {
        if let Some(existing) = self.symbols.get(&id) {
            if lamport > existing.lamport
                || (lamport == existing.lamport && content_hash > existing.content_hash)
            {
                self.commit_node(id, lamport, content_hash, dependencies, body, is_delete);
                return true;
            }
            false
        } else {
            self.commit_node(id, lamport, content_hash, dependencies, body, is_delete);
            true
        }
    }

    fn commit_node(
        &mut self,
        id: String,
        lamport: u64,
        content_hash: u64,
        dependencies: Vec<String>,
        body: String,
        is_delete: bool,
    ) {
        if is_delete {
            self.symbols.insert(
                id.clone(),
                SymbolNode {
                    id,
                    lamport,
                    content_hash,
                    dependencies,
                    body: String::new(),
                    is_tombstone: true,
                    deleted_tick: self.current_tick,
                },
            );
        } else {
            self.symbols.insert(
                id.clone(),
                SymbolNode {
                    id,
                    lamport,
                    content_hash,
                    dependencies,
                    body,
                    is_tombstone: false,
                    deleted_tick: 0,
                },
            );
        }
    }

    pub fn prune_tombstones(&mut self) {
        let current_tick = self.current_tick;
        let ttl = self.tombstone_ttl;
        self.symbols.retain(|_, node| {
            if node.is_tombstone {
                current_tick.saturating_sub(node.deleted_tick) <= ttl
            } else {
                true
            }
        });
    }

    pub fn materialize_active_source(&self) -> String {
        let mut keys: Vec<&String> = self.symbols.keys().collect();
        keys.sort();
        let mut out = String::new();
        for k in keys {
            let node = self.symbols.get(k).unwrap();
            if !node.is_tombstone {
                out.push_str(&node.body);
                out.push('\n');
            }
        }
        out
    }

    pub fn compute_hash(content: &str) -> u64 {
        xxh64(content.as_bytes(), 0)
    }
}
