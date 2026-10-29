use pseudo_harness::tui::TuiState;
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
    if args.len() > 1 && args[1] == "transpile" {
        if args.len() > 2 {
            let code = std::fs::read_to_string(&args[2])
                .unwrap_or_else(|_| "f main () [ r 0 ]".to_string());
            let transpiler = Transpiler::new();
            match transpiler.transpile(&code) {
                Ok(rs) => println!("{}", rs),
                Err(e) => eprintln!("Transpile error: {}", e),
            }
            return Ok(());
        }
    }

    // Terminal UI Execution
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = TuiState::new();
    let _engine = EngineClient::default_client();
    let transpiler = Transpiler::new();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(f.size());

            let title = Paragraph::new(" pseudo-harness v3.0 - AI TUI & Transpiler ")
                .block(Block::default().borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
            f.render_widget(title, chunks[0]);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let output = Paragraph::new(state.output_view.as_str())
                .block(Block::default().title("AI / Transpile View").borders(Borders::ALL));
            f.render_widget(output, main_chunks[0]);

            let transpiled = Paragraph::new(state.transpiled_code.as_str())
                .block(Block::default().title("Generated Rust").borders(Borders::ALL));
            f.render_widget(transpiled, main_chunks[1]);

            let input = Paragraph::new(state.input_buffer.as_str())
                .block(Block::default().title("Input Prompt / pseudo-lang").borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)));
            f.render_widget(input, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                state.handle_key(key.code);
                if key.code == KeyCode::Enter && !state.input_buffer.is_empty() {
                    match transpiler.transpile(&state.input_buffer) {
                        Ok(rs) => {
                            state.transpiled_code = rs;
                            state.output_view = format!("Transpiled pseudo-lang to Rust successfully.");
                        }
                        Err(e) => {
                            state.output_view = format!("Transpile error: {}", e);
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
