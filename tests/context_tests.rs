use pseudo_harness::crdt::SymbolMapCRDT;
use pseudo_harness::projector::ContextProjector;

#[test]
fn test_context_projector_budget_ceiling() {
    let mut crdt = SymbolMapCRDT::new(10);
    crdt.advance_tick();

    let body = "f process_data (x:n) [ r (+ x 1) ]".to_string();
    let hash = SymbolMapCRDT::compute_hash(&body);
    crdt.apply_patch("math.process".into(), 1, hash, vec![], body, false);

    let projector = ContextProjector::new(1200); // 1,200 token limit
    let slice = projector.build_slice(&crdt, "math.process", "").unwrap();

    assert!(slice.contains("TOOL SIGNATURES"));
    assert!(slice.contains("math.process"));
    assert!(ContextProjector::estimate_tokens(&slice) <= 1200);
}

#[test]
fn test_context_projector_error_frame_injection() {
    let mut crdt = SymbolMapCRDT::new(10);
    crdt.advance_tick();

    let projector = ContextProjector::new(1200);
    let slice = projector.build_slice(&crdt, "nonexistent", "VM TRAP: Undefined symbol").unwrap();

    assert!(slice.contains("<error>"));
    assert!(slice.contains("VM TRAP: Undefined symbol"));
}
