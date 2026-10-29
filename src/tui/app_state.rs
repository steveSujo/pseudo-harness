use crate::tui::completion::{CompletionItem, CompletionKind, SymbolIndex};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavMode {
    Normal,
    Insert,
    Inspect,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    FileTree,
    Editor,
    StreamPanel,
    Prompt,
}

pub struct AppState {
    pub mode: NavMode,
    pub focused_pane: FocusedPane,
    pub input_buffer: String,
    pub editor_buffer: String,
    pub output_view: String,
    pub transpiled_code: String,
    pub should_quit: bool,
    pub token_count: usize,
    pub token_ceiling: usize,

    // File Tree
    pub file_tree: Vec<String>,
    pub selected_file_idx: usize,

    // Drawers
    pub show_agenda: bool,
    pub show_repo_map: bool,
    pub show_diagnostics: bool,
    pub diagnostics_log: String,
    pub agenda_tasks: Vec<String>,
    pub repo_symbols: Vec<String>,

    // @ Completion
    pub symbol_index: SymbolIndex,
    pub completion_candidates: Vec<CompletionItem>,
    pub completion_selected: usize,
    pub show_completion: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let mut symbol_index = SymbolIndex::new();
        // Seed default symbols for @ completion
        symbol_index.add_item(CompletionItem {
            display_text: "@net/client.pseudo".into(),
            insert_text: "net/client.pseudo ".into(),
            kind: CompletionKind::File("src/net/client.pseudo".into()),
            token_cost: 45,
        });
        symbol_index.add_item(CompletionItem {
            display_text: "@net::client::fetch".into(),
            insert_text: "net::client::fetch ".into(),
            kind: CompletionKind::Function("net::client::fetch".into()),
            token_cost: 32,
        });
        symbol_index.add_item(CompletionItem {
            display_text: "@net::client::Config".into(),
            insert_text: "net::client::Config ".into(),
            kind: CompletionKind::Struct("net::client::Config".into()),
            token_cost: 20,
        });
        symbol_index.add_item(CompletionItem {
            display_text: "@storage::engine::Store".into(),
            insert_text: "storage::engine::Store ".into(),
            kind: CompletionKind::Struct("storage::engine::Store".into()),
            token_cost: 50,
        });
        symbol_index.add_item(CompletionItem {
            display_text: "@system::clock".into(),
            insert_text: "system::clock ".into(),
            kind: CompletionKind::Function("system::clock".into()),
            token_cost: 15,
        });
        symbol_index.add_item(CompletionItem {
            display_text: "@sqlite::query".into(),
            insert_text: "sqlite::query ".into(),
            kind: CompletionKind::Function("sqlite::query".into()),
            token_cost: 25,
        });

        Self {
            mode: NavMode::Normal,
            focused_pane: FocusedPane::Prompt,
            input_buffer: String::new(),
            editor_buffer: "// pseudo-lang active buffer\nf main () [\n    v x 42\n    r x\n]".into(),
            output_view: "Welcome to pseudo-harness v3.1 TUI Workbench\nPress 'i' for Insert, ':' for Command, 'v' for Inspect, '^Q' to Quit.\nType '@' in Insert mode for in-context symbol completion.".into(),
            transpiled_code: "// Transpiled Rust Output\npub fn main() -> i64 {\n    let x = 42;\n    x\n}".into(),
            should_quit: false,
            token_count: 340,
            token_ceiling: 1200,

            file_tree: vec![
                "src/main.rs".into(),
                "src/lib.rs".into(),
                "src/crdt.rs".into(),
                "src/sandbox.rs".into(),
                "src/vm.rs".into(),
                "src/tui/mod.rs".into(),
                ".pseudo/main.pir".into(),
                ".pseudo/todo.json".into(),
                "Cargo.toml".into(),
                "README.md".into(),
                "SPEC.md".into(),
            ],
            selected_file_idx: 0,

            show_agenda: false,
            show_repo_map: false,
            show_diagnostics: false,
            diagnostics_log: "[VM OK] Virtual Machine ready\n[SANDBOX] Jailed to <repo>/.pseudo/\n[CRDT] Lamport tick: 142 | 0 conflicts\n[ENGINE] 3.2B Ternary | VRAM: 1.4/6.0 GB | 42 t/s\n[TOKEN] 340/1200 tokens (Safe headroom)".into(),
            agenda_tasks: vec![
                "[DONE] Epic 1: Core VM & CRDT Runtime".into(),
                "[DONE] Epic 2: Bidirectional Transpiler".into(),
                "[WIP] Epic 3: Interactive TUI v3.1 & Keymaps".into(),
                "[TODO] Epic 4: psudo-engine VRAM Streaming".into(),
            ],
            repo_symbols: vec![
                "crdt::SymbolMapCRDT".into(),
                "sandbox::Sandbox".into(),
                "vm::VirtualMachine".into(),
                "net::client::fetch".into(),
                "storage::engine::Store".into(),
            ],

            symbol_index,
            completion_candidates: Vec::new(),
            completion_selected: 0,
            show_completion: false,
        }
    }

    pub fn set_mode(&mut self, mode: NavMode) {
        self.mode = mode;
        if mode != NavMode::Insert {
            self.show_completion = false;
        }
    }

    pub fn handle_input(&mut self, c: char) {
        self.input_buffer.push(c);
        self.update_completion_query();
    }

    pub fn update_completion_query(&mut self) {
        if self.mode == NavMode::Insert {
            if let Some(at_idx) = self.input_buffer.rfind('@') {
                let query = &self.input_buffer[at_idx + 1..];
                if !query.contains(' ') {
                    self.completion_candidates = self.symbol_index.search(query);
                    self.show_completion = !self.completion_candidates.is_empty();
                    self.completion_selected = 0;
                    return;
                }
            }
        }
        self.show_completion = false;
    }

    pub fn cycle_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::FileTree => FocusedPane::Editor,
            FocusedPane::Editor => FocusedPane::StreamPanel,
            FocusedPane::StreamPanel => FocusedPane::Prompt,
            FocusedPane::Prompt => FocusedPane::FileTree,
        };
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        self.handle_key_event(KeyEvent::new(code, KeyModifiers::empty()));
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        // Universal control commands from any mode
        if event.modifiers.contains(KeyModifiers::CONTROL) {
            match event.code {
                KeyCode::Char('q') | KeyCode::Char('c') => {
                    self.should_quit = true;
                    return;
                }
                KeyCode::Char('b') => {
                    self.show_agenda = !self.show_agenda;
                    return;
                }
                KeyCode::Char('m') => {
                    self.show_repo_map = !self.show_repo_map;
                    return;
                }
                KeyCode::Char('d') => {
                    self.show_diagnostics = !self.show_diagnostics;
                    return;
                }
                _ => {}
            }
        }

        // Active drawers close on Esc
        if event.code == KeyCode::Esc && (self.show_agenda || self.show_repo_map || self.show_diagnostics) {
            self.show_agenda = false;
            self.show_repo_map = false;
            self.show_diagnostics = false;
            return;
        }

        // If completion popup is visible in Insert mode
        if self.show_completion && self.mode == NavMode::Insert {
            match event.code {
                KeyCode::Down | KeyCode::Tab => {
                    if !self.completion_candidates.is_empty() {
                        self.completion_selected = (self.completion_selected + 1) % self.completion_candidates.len();
                    }
                    return;
                }
                KeyCode::Up | KeyCode::BackTab => {
                    if !self.completion_candidates.is_empty() {
                        if self.completion_selected == 0 {
                            self.completion_selected = self.completion_candidates.len() - 1;
                        } else {
                            self.completion_selected -= 1;
                        }
                    }
                    return;
                }
                KeyCode::Enter => {
                    if !self.completion_candidates.is_empty() {
                        let selected = &self.completion_candidates[self.completion_selected];
                        if let Some(at_idx) = self.input_buffer.rfind('@') {
                            self.input_buffer.truncate(at_idx);
                            self.input_buffer.push('@');
                            self.input_buffer.push_str(&selected.insert_text);
                        }
                        self.show_completion = false;
                    }
                    return;
                }
                KeyCode::Esc => {
                    self.show_completion = false;
                    return;
                }
                _ => {}
            }
        }

        match self.mode {
            NavMode::Normal => match event.code {
                KeyCode::Char('q') => {
                    self.should_quit = true;
                }
                KeyCode::Char('i') => {
                    self.mode = NavMode::Insert;
                }
                KeyCode::Char(':') => {
                    self.mode = NavMode::Command;
                    self.input_buffer.clear();
                }
                KeyCode::Char('v') | KeyCode::Char(' ') => {
                    self.mode = NavMode::Inspect;
                }
                KeyCode::Tab => {
                    self.cycle_pane();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.focused_pane == FocusedPane::FileTree && !self.file_tree.is_empty() {
                        self.selected_file_idx = (self.selected_file_idx + 1) % self.file_tree.len();
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.focused_pane == FocusedPane::FileTree && !self.file_tree.is_empty() {
                        if self.selected_file_idx == 0 {
                            self.selected_file_idx = self.file_tree.len() - 1;
                        } else {
                            self.selected_file_idx -= 1;
                        }
                    }
                }
                KeyCode::Enter => {
                    if !self.input_buffer.is_empty() {
                        let input = self.input_buffer.clone();
                        self.output_view = format!("Processed: {}", input);
                        self.input_buffer.clear();
                    } else if self.focused_pane == FocusedPane::FileTree && !self.file_tree.is_empty() {
                        let selected_file = &self.file_tree[self.selected_file_idx];
                        self.output_view = format!("Opened file: {}", selected_file);
                    }
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                _ => {}
            },
            NavMode::Insert => match event.code {
                KeyCode::Esc => {
                    self.mode = NavMode::Normal;
                    self.show_completion = false;
                }
                KeyCode::Char(c) => {
                    self.handle_input(c);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    self.update_completion_query();
                }
                KeyCode::Enter => {
                    let input = self.input_buffer.clone();
                    self.output_view = format!("Processed: {}", input);
                    self.input_buffer.clear();
                    self.show_completion = false;
                }
                _ => {}
            },
            NavMode::Inspect => match event.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                    self.mode = NavMode::Normal;
                }
                _ => {}
            },
            NavMode::Command => match event.code {
                KeyCode::Esc => {
                    self.mode = NavMode::Normal;
                    self.input_buffer.clear();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Enter => {
                    let cmd = self.input_buffer.trim().to_string();
                    self.input_buffer.clear();
                    self.mode = NavMode::Normal;
                    match cmd.as_str() {
                        "q" | "quit" => self.should_quit = true,
                        "w" | "save" => self.output_view = "Buffer saved to disk.".into(),
                        "transpile" => self.output_view = "Triggered transpilation pass.".into(),
                        "clear" => {
                            self.output_view.clear();
                            self.transpiled_code.clear();
                        }
                        other if !other.is_empty() => {
                            self.output_view = format!("Command executed: :{}", other);
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.mode, NavMode::Normal);
        assert!(!state.should_quit);
        assert!(!state.file_tree.is_empty());
    }

    #[test]
    fn test_ctrl_q_quits_from_insert_mode() {
        let mut state = AppState::new();
        state.set_mode(NavMode::Insert);
        state.handle_input('x');
        state.handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
        assert!(state.should_quit);
    }

    #[test]
    fn test_ctrl_b_toggles_agenda() {
        let mut state = AppState::new();
        assert!(!state.show_agenda);
        state.handle_key_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL));
        assert!(state.show_agenda);
        state.handle_key_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::CONTROL));
        assert!(!state.show_agenda);
    }

    #[test]
    fn test_at_reference_completion_flow() {
        let mut state = AppState::new();
        state.set_mode(NavMode::Insert);
        state.handle_input('@');
        assert!(state.show_completion);
        assert!(!state.completion_candidates.is_empty());

        state.handle_input('n');
        state.handle_input('e');
        state.handle_input('t');
        assert!(state.show_completion);

        state.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty()));
        assert!(!state.show_completion);
        assert!(state.input_buffer.starts_with('@'));
    }
}
