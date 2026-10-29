use pseudo_harness::tui::TuiState;
use crossterm::event::KeyCode;

#[test]
fn test_tui_state_handling() {
    let mut state = TuiState::new();
    assert!(!state.should_quit);

    state.handle_key(KeyCode::Char('a'));
    state.handle_key(KeyCode::Char('b'));
    assert_eq!(state.input_buffer, "ab");

    state.handle_key(KeyCode::Backspace);
    assert_eq!(state.input_buffer, "a");

    state.handle_key(KeyCode::Enter);
    assert_eq!(state.input_buffer, "");
    assert!(state.output_view.contains("Processed: a"));

    state.handle_key(KeyCode::Char('q'));
    assert!(state.should_quit);
}
