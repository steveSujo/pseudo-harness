use pseudo_harness::crdt::SymbolMapCRDT;

#[test]
fn test_crdt_patch_precedence_and_hash_tiebreaker() {
    let mut crdt = SymbolMapCRDT::new(10);
    crdt.advance_tick(); // tick 1

    let hash1 = SymbolMapCRDT::compute_hash("body_v1");
    let applied1 = crdt.apply_patch(
        "net.http.fetch".into(),
        1,
        hash1,
        vec![],
        "body_v1".into(),
        false,
    );
    assert!(applied1);

    // Lower lamport should be rejected
    let applied_low = crdt.apply_patch(
        "net.http.fetch".into(),
        0,
        hash1 + 100,
        vec![],
        "old_body".into(),
        false,
    );
    assert!(!applied_low);
    assert_eq!(crdt.symbols.get("net.http.fetch").unwrap().body, "body_v1");

    // Equal lamport, lower hash should be rejected
    let hash_lower = hash1 - 1;
    let applied_eq_low = crdt.apply_patch(
        "net.http.fetch".into(),
        1,
        hash_lower,
        vec![],
        "low_hash_body".into(),
        false,
    );
    assert!(!applied_eq_low);

    // Equal lamport, higher hash should win
    let hash_higher = hash1 + 10;
    let applied_eq_high = crdt.apply_patch(
        "net.http.fetch".into(),
        1,
        hash_higher,
        vec![],
        "body_v2".into(),
        false,
    );
    assert!(applied_eq_high);
    assert_eq!(crdt.symbols.get("net.http.fetch").unwrap().body, "body_v2");
}

#[test]
fn test_tombstone_pruning_and_lexicographical_materialization() {
    let mut crdt = SymbolMapCRDT::new(5); // TTL = 5 ticks

    crdt.advance_tick(); // tick 1
    crdt.apply_patch("b_symbol".into(), 1, 100, vec![], "fn b() {}".into(), false);
    crdt.apply_patch("a_symbol".into(), 1, 100, vec![], "fn a() {}".into(), false);

    let materialized = crdt.materialize_active_source();
    assert!(materialized.contains("fn a() {}"));
    assert!(materialized.contains("fn b() {}"));

    // Delete a_symbol at tick 1
    crdt.apply_patch("a_symbol".into(), 2, 200, vec![], "".into(), true);
    assert!(crdt.symbols.get("a_symbol").unwrap().is_tombstone);

    // Advance ticks so age > 5 (deleted at tick 1, advance 6 ticks -> tick 7, age = 6 > 5)
    for _ in 0..6 {
        crdt.advance_tick();
    }
    assert!(!crdt.symbols.contains_key("a_symbol"));
}
