use std::path::PathBuf;
use super::fuzzy::fuzzy_score;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    File(PathBuf),
    Function(String),
    Struct(String),
    Module(String),
}

#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub display_text: String,
    pub insert_text: String,
    pub kind: CompletionKind,
    pub token_cost: usize,
}

pub struct SymbolIndex {
    items: Vec<CompletionItem>,
}

impl SymbolIndex {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add_item(&mut self, item: CompletionItem) {
        self.items.push(item);
    }

    pub fn search(&self, query: &str) -> Vec<CompletionItem> {
        if query.is_empty() {
            return self.items.clone();
        }

        let mut scored_items: Vec<(i32, CompletionItem)> = Vec::new();

        for item in &self.items {
            let score_display = fuzzy_score(query, &item.display_text);
            let score_insert = fuzzy_score(query, &item.insert_text);
            let max_score = match (score_display, score_insert) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (Some(a), None) => Some(a),
                (None, Some(b)) => Some(b),
                (None, None) => None,
            };

            if let Some(score) = max_score {
                scored_items.push((score, item.clone()));
            }
        }

        scored_items.sort_by(|a, b| b.0.cmp(&a.0));

        scored_items.into_iter().map(|(_, item)| item).collect()
    }
}
