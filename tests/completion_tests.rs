use pseudo_harness::tui::completion::{fuzzy_score, CompletionItem, CompletionKind, SymbolIndex};

#[test]
fn test_fuzzy_scoring() {
    let score = fuzzy_score("net", "net::client::fetch");
    assert!(score.is_some());

    let no_score = fuzzy_score("xyz", "net::client::fetch");
    assert!(no_score.is_none());
}

#[test]
fn test_symbol_index_search() {
    let mut index = SymbolIndex::new();
    index.add_item(CompletionItem {
        display_text: "@net::client::fetch".into(),
        insert_text: "net::client::fetch ".into(),
        kind: CompletionKind::Function("net::client::fetch".into()),
        token_cost: 15,
    });

    let results = index.search("net");
    assert!(!results.is_empty());
}
