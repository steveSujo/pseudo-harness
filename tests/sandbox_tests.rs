use pseudo_harness::sandbox::Sandbox;
use tempfile::TempDir;

#[test]
fn test_sandbox_path_jail_and_traversal_rejection() {
    let tmp = TempDir::new().unwrap();
    let sandbox = Sandbox::new(tmp.path()).unwrap();

    // Valid path inside sandbox
    let valid_res = sandbox.validate_path("main.pir");
    assert!(valid_res.is_ok());

    // Path traversal attempt via ".."
    let traversal_res = sandbox.validate_path("../outside.txt");
    assert!(traversal_res.is_err());
    assert!(traversal_res.unwrap_err().contains("Security violation"));
}

#[test]
fn test_atomic_write_and_rename() {
    let tmp = TempDir::new().unwrap();
    let sandbox = Sandbox::new(tmp.path()).unwrap();

    let written = sandbox.atomic_write("main.pir", "v test_symbol = 1").unwrap();
    assert!(written.exists());

    let content = std::fs::read_to_string(&written).unwrap();
    assert_eq!(content, "v test_symbol = 1");

    // Verify temp file does not remain
    let tmp_file = written.with_extension("pir.tmp");
    assert!(!tmp_file.exists());
}
