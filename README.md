# pseudo-harness

Autonomous Rust TUI application and runtime harness for the **Psudo Ecosystem**. It assists users in authoring `pseudo-lang` code powered by the local `psudo-engine` AI model, enforcing strict token budget limits, and transpiling `pseudo-lang` into idiomatic, safe production Rust code.

---

## Architecture

- **Terminal UI (`src/tui.rs`):** Ratatui/Crossterm-based interactive terminal interface with real-time prompt input, AI view, and generated Rust preview panes.
- **AI Model Client (`src/engine_client.rs`):** IPC client interfacing with `psudo-engine`, enforcing the strict 1,200-token operational ceiling.
- **Transpiler (`src/transpiler.rs`):** Bidirectional AST parser and code generator converting `pseudo-lang` opcodes (`v`, `=`, `?`, `w`, `@rd`, `@wt`) into production Rust source code.
- **Core VM & CRDT (`src/crdt.rs`, `src/sandbox.rs`, `src/vm.rs`):** LWW-Set Symbol Map CRDT, path-jailed filesystem sandbox, and pIR execution virtual machine.

---

## Usage

### 1. Run the Interactive TUI
```bash
cargo run
```

### 2. Transpile a pseudo-lang file directly via CLI
```bash
cargo run -- transpile path/to/source.pseudo
```

---

## Testing & Verification

Run the full test suite:
```bash
cargo test --all-targets
```
