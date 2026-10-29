use crate::tui::app_state::{AppState, NavMode};
use crossterm::event::KeyCode;

pub fn handle_key_binding(state: &mut AppState, code: KeyCode) {
    match state.mode {
        NavMode::Normal => match code {
            KeyCode::Char('q') | KeyCode::Esc => {
                state.should_quit = true;
            }
            KeyCode::Char('i') => {
                state.set_mode(NavMode::Insert);
            }
            KeyCode::Char(':') | KeyCode::Char('/') => {
                state.set_mode(NavMode::Command);
                state.input_buffer.clear();
            }
            KeyCode::Char('v') => {
                state.set_mode(NavMode::Inspect);
            }
            KeyCode::Enter
                if !state.input_buffer.is_empty() => {
                    let input = state.input_buffer.clone();
                    state.output_view = format!("Processed: {}", input);
                    state.input_buffer.clear();
                }
            KeyCode::Char(c) => {
                state.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                state.input_buffer.pop();
            }
            _ => {}
        },
        NavMode::Insert => match code {
            KeyCode::Esc => {
                state.set_mode(NavMode::Normal);
            }
            KeyCode::Char(c) => {
                state.handle_input(c);
            }
            KeyCode::Backspace => {
                state.input_buffer.pop();
            }
            KeyCode::Enter => {
                state.output_view = format!("Processed: {}", state.input_buffer);
                state.input_buffer.clear();
                state.set_mode(NavMode::Normal);
            }
            _ => {}
        },
        NavMode::Inspect => match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                state.set_mode(NavMode::Normal);
            }
            _ => {}
        },
        NavMode::Command => match code {
            KeyCode::Esc => {
                state.set_mode(NavMode::Normal);
                state.input_buffer.clear();
            }
            KeyCode::Enter => {
                let cmd = state.input_buffer.clone();
                state.input_buffer.clear();
                if cmd == "q" || cmd == "quit" {
                    state.should_quit = true;
                } else if !cmd.is_empty() {
                    state.output_view = format!("Command executed: {}", cmd);
                }
                state.set_mode(NavMode::Normal);
            }
            KeyCode::Char(c) => {
                state.input_buffer.push(c);
            }
            KeyCode::Backspace => {
                state.input_buffer.pop();
            }
            _ => {}
        },
    }
}
