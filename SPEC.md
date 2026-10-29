# pseudo-harness v3.0 Specification

## 1. Overview & System Role
`pseudo-harness` is a Rust Terminal User Interface (TUI) application designed to assist operators in authoring `pseudo-lang` code powered by the local `psudo-engine` AI model. It bridges natural language operator steering with deterministic Intermediate Representation (`pIR`) execution and automated transpilation to production Rust.

---

## 2. Component Specifications

### A. Terminal User Interface (`src/tui.rs`, `src/main.rs`)
- Built with `ratatui` and `crossterm`.
- Layout: Three-panel split (Header, AI/Transpile view and Generated Rust preview, Input prompt).
- Keyboard event handling (`Esc`/`q` to quit, `Enter` to process input and trigger transpilation/engine query).

### B. AI Model Client (`src/engine_client.rs`)
- Interfaces with the local `psudo-engine` binary/IPC.
- Enforces strict **1,200-token operational ceiling** (within the 1,536 physical token window) to guarantee zero swapping on 6GB VRAM hardware.
- KV-cache prefix pinning for static tool contracts and core invariants.

### C. pseudo-lang to Rust Transpiler (`src/transpiler.rs`)
- Parsers and AST generators for pseudo-lang opcodes (`v`, `=`, `?`, `w`, `@rd`, `@wt`).
- Deterministic code generation emitting idiomatic, safe Rust functions, structs, and expressions.
- CLI subcommand support: `cargo run -- transpile <file.pseudo>`.

### D. CRDT & Sandbox Runtime (`src/crdt.rs`, `src/sandbox.rs`, `src/vm.rs`)
- **LWW Symbol Map CRDT:** Lamport clock sequencing, content-hash tie breakers, and sliding-window tombstone GC ($\Delta K = 10$).
- **Path-Jailed Sandbox:** Rigorous filesystem path canonicalization rejecting traversal sequences (`..`).
- **Virtual Machine:** Executes pIR instructions and traps tool calls (`@x`).

---

## 3. Verification & Testing
- Comprehensive test suite covering TUI state, engine token budgeting, CRDT invariants, path jailing, and transpiler AST generation (`cargo test --all-targets`).
