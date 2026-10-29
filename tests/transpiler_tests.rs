use pseudo_harness::transpiler::Transpiler;

#[test]
fn test_transpile_simple_function() {
    let transpiler = Transpiler::new();
    let code = "f add(a:n, b:n) [ r (+ a b) ]";
    let rust_code = transpiler.transpile(code).unwrap();
    
    assert!(rust_code.contains("pub fn add(a: i32, b: i32) -> i32"));
    assert!(rust_code.contains("(a + b)"));
}

#[test]
fn test_transpile_variables_and_assignment() {
    let transpiler = Transpiler::new();
    let code = "f calc(x:n) [ v y:n = (* x 2); y = (+ y 1); r y ]";
    let rust_code = transpiler.transpile(code).unwrap();
    
    assert!(rust_code.contains("pub fn calc(x: i32) -> i32"));
    assert!(rust_code.contains("let y: i32 = (x * 2);"));
    assert!(rust_code.contains("y = (y + 1);"));
    assert!(rust_code.contains("y"));
}

#[test]
fn test_transpile_conditional() {
    let transpiler = Transpiler::new();
    let code = "f check(x:n) [ ? (> x 10) [ r 1 ] [ r 0 ] ]";
    let rust_code = transpiler.transpile(code).unwrap();
    
    assert!(rust_code.contains("pub fn check(x: i32) -> i32"));
    assert!(rust_code.contains("if (x > 10) {"));
    assert!(rust_code.contains("1"));
    assert!(rust_code.contains("} else {"));
    assert!(rust_code.contains("0"));
}

#[test]
fn test_transpile_while_loop() {
    let transpiler = Transpiler::new();
    let code = "f count(x:n) [ w (< x 10) [ x = (+ x 1); ]; r x ]";
    let rust_code = transpiler.transpile(code).unwrap();
    
    assert!(rust_code.contains("pub fn count(x: i32) -> i32"));
    assert!(rust_code.contains("while (x < 10) {"));
    assert!(rust_code.contains("x = (x + 1);"));
    assert!(rust_code.contains("x"));
}

#[test]
fn test_transpile_storage_ops() {
    let transpiler = Transpiler::new();
    let code = r#"f query_db(path:s) [ v res = @rd "sqlite" "query" {db: path, query: "SELECT *"}; r res ]"#;
    let rust_code = transpiler.transpile(code).unwrap();
    
    assert!(rust_code.contains("pub fn query_db(path: String) -> i32"));
    assert!(rust_code.contains("sandbox.dispatch(\"sqlite\", \"query\", &serde_json::json!({db: path, query: \"SELECT *\"}))"));
    assert!(rust_code.contains("res"));
}
