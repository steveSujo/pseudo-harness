use crate::tui::app_state::AppState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_binding(state: &mut AppState, code: KeyCode) {
    state.handle_key_event(KeyEvent::new(code, KeyModifiers::empty()));
}

pub fn handle_key_event_binding(state: &mut AppState, event: KeyEvent) {
    state.handle_key_event(event);
}
