use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use pseudo_harness::crdt::SymbolMapCRDT;
use pseudo_harness::engine_client::EngineClient;
use pseudo_harness::sandbox::Sandbox;
use pseudo_harness::server::{create_router, ServerState};
use pseudo_harness::vm::VirtualMachine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut port = 3000;
    let mut host = "127.0.0.1".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(3000);
                    i += 1;
                }
            }
            "--host" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("pseudo-harness v3.1 - Localhost WebApp & Autonomous Runtime");
                println!();
                println!("USAGE:");
                println!("    pseudo-harness [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    -p, --port <PORT>    Port to listen on [default: 3000]");
                println!("        --host <HOST>    Host address to bind [default: 127.0.0.1]");
                println!("    -h, --help           Print help information");
                println!("    -v, --version        Print version information");
                return Ok(());
            }
            "--version" | "-v" => {
                println!("pseudo-harness 3.1.0");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    // Initialize Harness State
    let crdt = Arc::new(Mutex::new(SymbolMapCRDT::new(10)));
    
    // Ensure sandbox root exists
    let sandbox_path = std::path::PathBuf::from(".pseudo/sandbox");
    let _ = std::fs::create_dir_all(&sandbox_path);
    let sandbox = Sandbox::new(&sandbox_path)
        .unwrap_or_else(|_| Sandbox::new(std::env::temp_dir()).unwrap());
    let vm = Arc::new(Mutex::new(VirtualMachine::new(sandbox)));
    let engine = Arc::new(EngineClient::default_client());
    let token_count = Arc::new(Mutex::new(340));

    let state = ServerState {
        crdt,
        vm,
        engine,
        token_count,
        token_ceiling: 1200,
    };

    let app = create_router(state, Some("static"));
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    println!("============================================================");
    println!("   🚀 pseudo-harness v3.1 WebApp Server Active");
    println!("   🌐 Local Workbench: http://{}", addr);
    println!("   ⚙️  API Endpoints:   http://{}/api/status", addr);
    println!("   🧠 VRAM Ceiling:    1,200 tokens (GTX 1660 Ti profile)");
    println!("============================================================");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
