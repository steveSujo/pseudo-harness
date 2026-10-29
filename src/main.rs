use pseudo_harness::tui::{AppState, NavMode};
use pseudo_harness::engine_client::EngineClient;
use pseudo_harness::transpiler::Transpiler;
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
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "transpile"
        && args.len() > 2 {
            let code = std::fs::read_to_string(&args[2])
                .unwrap_or_else(|_| "f main () [ r 0 ]".to_string());
            let transpiler = Transpiler::new();
            match transpiler.transpile(&code) {
                Ok(rs) => println!("{}", rs),
                Err(e) => eprintln!("Transpile error: {}", e),
            }
            return Ok(());
        }

    // Terminal UI Execution
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new();
    let _engine = EngineClient::default_client();
    let transpiler = Transpiler::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // Status bar
                    Constraint::Length(3), // Header
                    Constraint::Min(5),    // Main content
                    Constraint::Length(3), // Input prompt
                ])
                .split(f.size());

            // 1. Status Bar
            let token_pct = (state.token_count * 100) / state.token_ceiling;
            let gauge_color = if token_pct > 95 {
                Color::Red
            } else if token_pct > 75 {
                Color::Yellow
            } else {
                Color::Green
            };
            let status_text = format!(
                " [● ENGINE: 3.2B TERNARY | 42 t/s]  [VRAM: 1.4/6.0 GB]  [BUDG: {}/{} ({}%)]",
                state.token_count, state.token_ceiling, token_pct
            );
            let status_bar = Paragraph::new(status_text)
                .style(Style::default().fg(gauge_color).bg(Color::Black));
            f.render_widget(status_bar, chunks[0]);

            // 2. Header
            let title = Paragraph::new(" pseudo-harness v3.1 - AI TUI & Transpiler ")
                .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
            f.render_widget(title, chunks[1]);

            // 3. Main Split
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[2]);

            let output = Paragraph::new(state.output_view.as_str())
                .block(Block::default().title("AI / Transpile View").borders(Borders::ALL));
            f.render_widget(output, main_chunks[0]);

            let transpiled = Paragraph::new(state.transpiled_code.as_str())
                .block(Block::default().title("Generated Rust").borders(Borders::ALL));
            f.render_widget(transpiled, main_chunks[1]);

            // 4. Input Prompt
            let mode_str = match state.mode {
                NavMode::Normal => "[NORMAL]",
                NavMode::Insert => "[INSERT]",
                NavMode::Inspect => "[INSPECT]",
                NavMode::Command => "[COMMAND]",
            };
            let prompt_title = format!(" Input Prompt / pseudo-lang {} ", mode_str);
            let input = Paragraph::new(state.input_buffer.as_str())
                .block(Block::default().title(prompt_title).borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)));
            f.render_widget(input, chunks[3]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') && state.mode == NavMode::Normal {
                    state.should_quit = true;
                } else if key.code == KeyCode::Tab {
                    // Cycle mode
                    let next_mode = match state.mode {
                        NavMode::Normal => NavMode::Insert,
                        NavMode::Insert => NavMode::Inspect,
                        NavMode::Inspect => NavMode::Command,
                        NavMode::Command => NavMode::Normal,
                    };
                    state.set_mode(next_mode);
                } else {
                    state.handle_key(key.code);
                    if key.code == KeyCode::Enter && !state.input_buffer.is_empty() {
                        match transpiler.transpile(&state.input_buffer) {
                            Ok(rs) => {
                                state.transpiled_code = rs;
                                state.output_view = "Transpiled pseudo-lang to Rust successfully.".to_string();
                                state.token_count = std::cmp::min(state.token_ceiling, state.token_count + state.input_buffer.len() / 4);
                            }
                            Err(e) => {
                                state.output_view = format!("Transpile error: {}", e);
                            }
                        }
                    }
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
