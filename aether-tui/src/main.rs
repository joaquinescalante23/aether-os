//! Aether-TUI (The Aether Monitor)
//!
//! Created by Joaquín Escalante (https://github.com/joaquinescalante23)

use chronos_proto::chronos_kernel_client::ChronosKernelClient;
use chronos_proto::{ListAgentsRequest, PauseAgentRequest, ResumeAgentRequest, StopAgentRequest, AgentSummary};
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

/// App state for managing real-time agent data.
struct App {
    client: ChronosKernelClient<Channel>,
    agents: Vec<AgentSummary>,
    list_state: ListState,
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
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.agents.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 { self.agents.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
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

    // 2. Initialize App State (Connecting to Kernel)
    let addr = "http://[::1]:50051";
    let mut app = match App::new(addr).await {
        Ok(app) => app,
        Err(e) => {
            // Restore terminal before exit
            disable_raw_mode()?;
            execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
            eprintln!("Failed to connect to AetherOS Kernel at {}: {}", addr, e);
            return Ok(());
        }
    };

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
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.previous(),
                    KeyCode::Char('p') => app.pause_selected().await,
                    KeyCode::Char('r') => app.resume_selected().await,
                    KeyCode::Char('k') => app.stop_selected().await,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.refresh_agents().await;
            last_tick = std::time::Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Main Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(size);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" AetherOS Kernel Monitor ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("| Connected: [::1]:50051"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Content Grid
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
        ])
        .split(chunks[1]);

    // 1. Process List (Real Data)
    let items: Vec<ListItem> = app.agents.iter().map(|a| {
        let state_str = match a.state {
            0 => "PENDING",
            1 => "RUNNING",
            2 => "SUSPENDED",
            3 => "TERMINATED",
            4 => "ERROR",
            _ => "UNKNOWN",
        };
        let color = match a.state {
            1 => Color::Green,
            2 => Color::Yellow,
            4 => Color::Red,
            _ => Color::White,
        };
        ListItem::new(format!("{:<8} | {:<10} | {:<10}", &a.agent_id[..8], a.name, state_str))
            .style(Style::default().fg(color))
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" PROCESS LIST ").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");
    f.render_stateful_widget(list, content_chunks[0], &mut app.list_state);

    // 2. Cognitive Stream (Placeholder for now, requires streaming integration)
    let selected_name = app.list_state.selected()
        .and_then(|i| app.agents.get(i))
        .map(|a| a.name.clone())
        .unwrap_or_else(|| "None".to_string());

    let stream = Paragraph::new(format!("Monitoring: {}\n\n(Cognitive Stream implementation in progress...)", selected_name))
        .block(Block::default().title(" 🧠 COGNITIVE STREAM ").borders(Borders::ALL));
    f.render_widget(stream, content_chunks[1]);

    // 3. Resources (Real Data)
    let stats_text = if let Some(i) = app.list_state.selected() {
        if let Some(agent) = app.agents.get(i) {
            let cost = agent.stats.as_ref().map(|s| s.cost_usd).unwrap_or(0.0);
            let tokens = agent.stats.as_ref().map(|s| s.tokens_consumed).unwrap_or(0);
            vec![
                Line::from(format!("Agent: {}", agent.name)),
                Line::from(format!("Cost: ${:.4}", cost)),
                Line::from(format!("Tokens: {}", tokens)),
            ]
        } else { vec![] }
    } else { vec![Line::from("No agent selected")] };

    let resources = Paragraph::new(stats_text)
        .block(Block::default().title(" 📊 RESOURCES ").borders(Borders::ALL));
    f.render_widget(resources, content_chunks[2]);

    // Footer
    let footer_text = if let Some(err) = &app.error_msg {
        format!(" ERROR: {} | Press 'Q' to quit", err)
    } else {
        " ⌨️  COMMANDS: [↑↓] Navigate | [P] Pause | [R] Resume | [K] Kill | [Q] Quit".to_string()
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

// Created by Joaquín Escalante (https://github.com/joaquinescalante23)
