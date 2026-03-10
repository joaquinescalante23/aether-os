//! Aether-TUI (The Aether Monitor)
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use chronos_proto::chronos_kernel_client::ChronosKernelClient;
use chronos_proto::{ListAgentsRequest, PauseAgentRequest, ResumeAgentRequest, StopAgentRequest, SendCommandRequest, AgentSummary};
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
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::{error::Error, io, time::Duration};
use tonic::transport::Channel;

pub mod chronos_proto {
    tonic::include_proto!("chronos");
}

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Editing,
}

/// App state for managing real-time agent data and interactivity.
struct App {
    client: ChronosKernelClient<Channel>,
    agents: Vec<AgentSummary>,
    list_state: ListState,
    input: String,
    input_mode: InputMode,
    should_quit: bool,
    error_msg: Option<String>,
}

impl App {
    async fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let client = ChronosKernelClient::connect(addr.to_string()).await?;
        Ok(Self {
            client,
            agents: Vec::new(),
            list_state: ListState::default(),
            input: String::new(),
            input_mode: InputMode::Normal,
            should_quit: false,
            error_msg: None,
        })
    }

    async fn refresh_agents(&mut self) {
        let request = tonic::Request::new(ListAgentsRequest {});
        match self.client.list_agents(request).await {
            Ok(response) => {
                self.agents = response.into_inner().agents;
                if self.list_state.selected().is_none() && !self.agents.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            Err(e) => self.error_msg = Some(format!("Refresh Error: {}", e)),
        }
    }

    fn selected_agent_id(&self) -> Option<String> {
        self.list_state.selected().and_then(|i| self.agents.get(i).map(|a| a.agent_id.clone()))
    }

    async fn send_command(&mut self) {
        if let Some(agent_id) = self.selected_agent_id() {
            let request = tonic::Request::new(SendCommandRequest {
                agent_id,
                content: self.input.drain(..).collect(),
            });
            let _ = self.client.send_command(request).await;
        }
        self.input_mode = InputMode::Normal;
    }

    async fn pause_selected(&mut self) {
        if let Some(agent_id) = self.selected_agent_id() {
            let request = tonic::Request::new(PauseAgentRequest { agent_id });
            let _ = self.client.pause_agent(request).await;
        }
    }

    async fn resume_selected(&mut self) {
        if let Some(agent_id) = self.selected_agent_id() {
            let request = tonic::Request::new(ResumeAgentRequest { 
                agent_id,
                checkpoint_id: "".to_string() 
            });
            let _ = self.client.resume_agent(request).await;
        }
    }

    async fn stop_selected(&mut self) {
        if let Some(agent_id) = self.selected_agent_id() {
            let request = tonic::Request::new(StopAgentRequest { agent_id });
            let _ = self.client.stop_agent(request).await;
        }
    }

    fn next(&mut self) {
        if self.agents.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => if i >= self.agents.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.agents.is_empty() { return; }
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { self.agents.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let addr = "http://[::1]:50051";
    let mut app = match App::new(addr).await {
        Ok(app) => app,
        Err(e) => {
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            eprintln!("Failed to connect to AetherOS Kernel at {}: {}", addr, e);
            return Ok(());
        }
    };

    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res { println!("{:?}", err) }
    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('i') | KeyCode::Char('/') => app.input_mode = InputMode::Editing,
                        KeyCode::Down => app.next(),
                        KeyCode::Up => app.previous(),
                        KeyCode::Char('p') => app.pause_selected().await,
                        KeyCode::Char('r') => app.resume_selected().await,
                        KeyCode::Char('k') => app.stop_selected().await,
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => app.send_command().await,
                        KeyCode::Char(c) => app.input.push(c),
                        KeyCode::Backspace => { app.input.pop(); },
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        _ => {}
                    },
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.refresh_agents().await;
            last_tick = std::time::Instant::now();
        }

        if app.should_quit { return Ok(()); }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Input/Footer
        ])
        .split(size);

    // 1. Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" AetherOS Kernel Monitor ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("| [::1]:50051"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // 2. Content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    let items: Vec<ListItem> = app.agents.iter().map(|a| {
        let state = match a.state { 1 => "RUN", 2 => "PAUSE", 4 => "ERR", _ => "IDLE" };
        let color = match a.state { 1 => Color::Green, 2 => Color::Yellow, 4 => Color::Red, _ => Color::White };
        ListItem::new(format!("{:<8} | {:<10} | {}", &a.agent_id[..8], a.name, state)).style(Style::default().fg(color))
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" PROCESSES ").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, content_chunks[0], &mut app.list_state);

    // Stream
    let selected_name = app.list_state.selected().and_then(|i| app.agents.get(i)).map(|a| a.name.as_str()).unwrap_or("None");
    let stream = Paragraph::new(format!("Monitoring Agent: {}\n\n(Stream connected...)", selected_name))
        .block(Block::default().title(" COGNITIVE STREAM ").borders(Borders::ALL));
    f.render_widget(stream, content_chunks[1]);

    // 3. Footer / Input
    let footer_text = match app.input_mode {
        InputMode::Normal => Line::from(vec![
            Span::styled(" [i] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Enter Command | "),
            Span::raw("[↑↓] Nav | [P] Pause | [R] Resume | [K] Kill | [Q] Quit"),
        ]),
        InputMode::Editing => Line::from(vec![
            Span::styled(" COMMAND > ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(&app.input),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]),
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).border_style(if app.input_mode == InputMode::Editing { Style::default().fg(Color::Green) } else { Style::default() }));
    f.render_widget(footer, chunks[2]);
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
