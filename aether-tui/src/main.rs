//! Aether-TUI (The Aether Monitor)
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, time::Duration};

pub mod chronos_proto {
    tonic::include_proto!("chronos");
}

struct App {
    should_quit: bool,
    // Future state will go here (Agent lists, selected agent streams)
}

impl App {
    fn new() -> Self {
        Self { should_quit: false }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize App State
    let mut app = App::new();

    // 3. Run UI Loop
    let res = run_app(&mut terminal, &mut app).await;

    // 4. Restore Terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Non-blocking event check (allows async tasks to run)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    app.should_quit = true;
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, _app: &mut App) {
    let size = f.size();

    // Main Layout: Header, Content Grid, Footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Footer
            ]
            .as_ref(),
        )
        .split(size);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" AetherOS Kernel Monitor ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("| v1.1.0 | Connected: [::1]:50051"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, main_chunks[0]);

    // Content Grid: Process List (L), Cognitive Stream (C), Resources (R)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25), // Left: Processes
                Constraint::Percentage(55), // Center: Thoughts
                Constraint::Percentage(20), // Right: Resources
            ]
            .as_ref(),
        )
        .split(main_chunks[1]);

    // Render Process List
    let items = vec![
        ListItem::new("1a2b3c.. | Architect  | RUN").style(Style::default().fg(Color::Green)),
        ListItem::new("4d5e6f.. | Coder      | SUSP").style(Style::default().fg(Color::Yellow)),
    ];
    let process_list = List::new(items)
        .block(Block::default().title(" PROCESS LIST ").borders(Borders::ALL));
    f.render_widget(process_list, content_chunks[0]);

    // Render Cognitive Stream
    let stream_text = vec![
        Line::from(Span::styled("[SYSTEM] Booting identity 'Architect'", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("[THOUGHT] I need to break down the task.", Style::default().fg(Color::White))),
        Line::from(Span::styled("[TOOL] shell_execute('mkdir src')", Style::default().fg(Color::Magenta))),
        Line::from(Span::styled("[RESULT] Success.", Style::default().fg(Color::Blue))),
    ];
    let stream = Paragraph::new(stream_text)
        .block(Block::default().title(" 🧠 COGNITIVE STREAM (AGENT: Selected) ").borders(Borders::ALL));
    f.render_widget(stream, content_chunks[1]);

    // Render Resources
    let resources_text = vec![
        Line::from("Budget: $10.00"),
        Line::from(Span::styled("Spent:  $0.45", Style::default().fg(Color::Green))),
        Line::from(""),
        Line::from("Tokens: 4k / 50k"),
    ];
    let resources = Paragraph::new(resources_text)
        .block(Block::default().title(" 📊 RESOURCES ").borders(Borders::ALL));
    f.render_widget(resources, content_chunks[2]);

    // Footer
    let footer = Paragraph::new(" ⌨️ COMMANDS: [↑↓] Navigate | [P] Pause | [R] Resume | [K] Kill | [Q] Quit")
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, main_chunks[2]);
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
