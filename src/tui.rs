use std::io;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

pub struct TuiState {
    pub input_buffer: String,
    pub output_view: String,
    pub transpiled_code: String,
    pub should_quit: bool,
}

impl TuiState {
    pub fn new() -> Self {
        Self {
            input_buffer: String::new(),
            output_view: "Welcome to pseudo-harness TUI v3.0".into(),
            transpiled_code: "".into(),
            should_quit: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Enter => {
                self.output_view = format!("Processed: {}", self.input_buffer);
                self.input_buffer.clear();
            }
            _ => {}
        }
    }
}
