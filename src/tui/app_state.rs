use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavMode {
    Normal,
    Insert,
    Inspect,
    Command,
}

pub struct AppState {
    pub mode: NavMode,
    pub input_buffer: String,
    pub output_view: String,
    pub transpiled_code: String,
    pub should_quit: bool,
    pub token_count: usize,
    pub token_ceiling: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: NavMode::Insert,
            input_buffer: String::new(),
            output_view: "Welcome to pseudo-harness v3.1 TUI".to_string(),
            transpiled_code: String::new(),
            should_quit: false,
            token_count: 0,
            token_ceiling: 1200,
        }
    }

    pub fn set_mode(&mut self, mode: NavMode) {
        self.mode = mode;
    }

    pub fn handle_input(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') if self.input_buffer.is_empty() || self.mode == NavMode::Normal => {
                self.should_quit = true;
            }
            KeyCode::Esc => {
                self.mode = NavMode::Normal;
            }
            KeyCode::Char('i') if self.mode == NavMode::Normal && self.input_buffer.is_empty() => {
                self.mode = NavMode::Insert;
            }
            KeyCode::Char('n') if self.mode == NavMode::Normal && self.input_buffer.is_empty() => {
                self.mode = NavMode::Normal;
            }
            KeyCode::Char('s') if self.mode == NavMode::Normal && self.input_buffer.is_empty() => {
                self.mode = NavMode::Inspect;
            }
            KeyCode::Char(':') if self.mode == NavMode::Normal && self.input_buffer.is_empty() => {
                self.mode = NavMode::Command;
                self.input_buffer.clear();
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Enter => {
                if self.mode == NavMode::Command {
                    if self.input_buffer == "q" || self.input_buffer == "quit" {
                        self.should_quit = true;
                    } else {
                        self.output_view = format!("Command executed: {}", self.input_buffer);
                    }
                    self.input_buffer.clear();
                    self.mode = NavMode::Normal;
                } else {
                    let input = self.input_buffer.clone();
                    if input == "q" {
                        self.should_quit = true;
                    } else {
                        self.output_view = format!("Processed: {}", input);
                        self.input_buffer.clear();
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.mode, NavMode::Insert);
        assert!(!state.should_quit);
        assert_eq!(state.token_ceiling, 1200);
    }

    #[test]
    fn test_set_mode() {
        let mut state = AppState::new();
        state.set_mode(NavMode::Normal);
        assert_eq!(state.mode, NavMode::Normal);
    }

    #[test]
    fn test_handle_input() {
        let mut state = AppState::new();
        state.handle_input('x');
        state.handle_input('y');
        assert_eq!(state.input_buffer, "xy");
    }
}
