# pseudo-harness

Autonomous Rust runtime and local Web Application workbench for the **Psudo Ecosystem**. It serves a browser-based SPA on `localhost:3000` to assist developers in authoring `pseudo-lang`, streaming `psudo-engine` AI model responses, tracking VRAM token budget limits, and verifying code execution inside a hermetic sandbox.

---

## Architecture

- **Localhost Web Server (`src/server.rs`):** Axum/Tokio HTTP & REST server serving the Svelte SPA workbench on `http://127.0.0.1:3000`.
- **Svelte SPA Workbench (`static/index.html`):** 4-region browser workbench:
  - **Region 1:** Workspace File & AST Tree Explorer.
  - **Region 2:** Active Editor for `pseudo-lang` code.
  - **Region 3:** Live AI assistant stream with `<talk>` reasoning and `<pir>` generated blocks.
  - **Region 4:** Operator prompt with real-time `@` symbol & file auto-complete popover.
  - **Header Telemetry:** Color-coded 1,200 token budget meter (Green / Amber / Red thresholds) and VRAM gauge.
  - **Drawers:** Agenda Drawer (`Ctrl+B`), Repo Map Drawer (`Ctrl+M`), Diagnostics Drawer (`Ctrl+D`).
- **AI Model Client (`src/engine_client.rs`):** Interfaces with local `psudo-engine` binary/IPC, enforcing the strict 1,200-token operational ceiling.
- **Core VM & CRDT (`src/crdt.rs`, `src/sandbox.rs`, `src/vm.rs`):** In-memory LWW-Set Symbol Map CRDT, path-jailed filesystem sandbox (`<repo>/.pseudo/`), and pIR virtual machine.

---

## Usage

### 1. Launch the Local WebApp
```bash
cargo run
```
Then open `http://127.0.0.1:3000` in any browser.

### 2. Custom Port / Host
```bash
cargo run -- --port 8080 --host 0.0.0.0
```

---

## Testing & Verification

Run the full integration test suite:
```bash
cargo test --all-targets
```
