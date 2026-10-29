use pseudo_harness::vm::VirtualMachine;
use pseudo_harness::sandbox::Sandbox;
use tempfile::TempDir;

#[test]
fn test_vm_system_clock_and_tool_dispatch() {
    let tmp = TempDir::new().unwrap();
    let sandbox = Sandbox::new(tmp.path()).unwrap();
    let vm = VirtualMachine::new(sandbox);

    let res = vm.dispatch("system", "clock", &serde_json::json!({}));
    assert!(res.ok);
    assert!(res.data.unwrap().get("epoch_ms").is_some());

    let unknown = vm.dispatch("unknown", "action", &serde_json::json!({}));
    assert!(!unknown.ok);
    assert!(unknown.error.unwrap().contains("Unknown tool contract"));
}

#[test]
fn test_vm_sqlite_security_jail() {
    let tmp = TempDir::new().unwrap();
    let sandbox = Sandbox::new(tmp.path()).unwrap();
    let vm = VirtualMachine::new(sandbox);

    // Traversal db path should fail
    let bad_res = vm.dispatch("sqlite", "query", &serde_json::json!({ "db": "../secret.db", "query": "SELECT * FROM users" }));
    assert!(!bad_res.ok);
    assert!(bad_res.error.unwrap().contains("Security violation"));

    // Valid db path inside sandbox
    let good_res = vm.dispatch("sqlite", "query", &serde_json::json!({ "db": "test.db", "query": "SELECT * FROM users" }));
    assert!(good_res.ok);
}
