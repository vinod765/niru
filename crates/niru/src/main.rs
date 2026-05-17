mod ipc;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as TermEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use ipc::IpcClient;
use niru_core::ipc::{Command, Event};

fn socket_path() -> String {
    let uid = unsafe { libc::getuid() };
    format!("/run/user/{}/nirud.sock", uid)
}

#[derive(Default)]
struct AppState {
    phase: String,
    remaining: u64,
    sessions_today: u32,
    streak: u32,
    status_line: String,
}

impl AppState {
    fn fmt_time(&self) -> String {
        let m = self.remaining / 60;
        let s = self.remaining % 60;
        format!("{:02}:{:02}", m, s)
    }

    fn phase_label(&self) -> &str {
        match self.phase.as_str() {
            "focus" => "FOCUS",
            "short_break" => "SHORT BREAK",
            "long_break" => "LONG BREAK",
            _ => "IDLE",
        }
    }

    fn phase_color(&self) -> Color {
        match self.phase.as_str() {
            "focus" => Color::Cyan,
            "short_break" => Color::Green,
            "long_break" => Color::Blue,
            _ => Color::DarkGray,
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let path = socket_path();
    let mut client = IpcClient::connect(&path).await.unwrap_or_else(|_| {
        eprintln!("Could not connect to nirud. Is the daemon running?");
        std::process::exit(1);
    });

    client.send(Command::Status).await?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::default();
    state.status_line = "Press s=start  p=pause  n=skip  q=quit".into();

    loop {
        // Drain all pending events from daemon
        loop {
            match client.event_rx.try_recv() {
                Ok(event) => apply_event(&mut state, event),
                Err(_) => break,
            }
        }

        terminal.draw(|f| render(f, &state))?;

        if event::poll(Duration::from_millis(200))? {
            if let TermEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('s') => {
                        client.send(Command::Start).await?;
                        state.status_line = "Running...".into();
                    }
                    KeyCode::Char('p') => {
                        client.send(Command::Pause).await?;
                        state.status_line = "Paused".into();
                    }
                    KeyCode::Char('n') => {
                        client.send(Command::Skip).await?;
                        state.status_line = "Skipped".into();
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn apply_event(state: &mut AppState, event: Event) {
    match event {
        Event::Tick { remaining, phase } => {
            state.remaining = remaining;
            state.phase = phase;
        }
        Event::StatusResponse {
            phase,
            remaining,
            streak,
            sessions_today,
        } => {
            state.phase = phase;
            state.remaining = remaining;
            state.streak = streak;
            state.sessions_today = sessions_today;
        }
        Event::SessionEnd { score } => {
            state.status_line = format!("Session done — score {}", score);
        }
        Event::BreakStart { duration } => {
            state.status_line = format!("Break — {}m", duration / 60);
        }
        Event::JournalPrompt => {
            state.status_line = "Journal: what did you work on?".into();
        }
        Event::Error { message } => {
            state.status_line = format!("Error: {}", message);
        }
    }
}

fn render(f: &mut ratatui::Frame, state: &AppState) {
    let area = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ])
        .split(area);

    // Phase label
    let phase = Paragraph::new(state.phase_label())
        .style(
            Style::default()
                .fg(state.phase_color())
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    f.render_widget(phase, chunks[0]);

    // Timer
    let timer = Paragraph::new(state.fmt_time())
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(timer, chunks[1]);

    // Stats
    let stats = Paragraph::new(Line::from(vec![
        Span::styled("Sessions: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            state.sessions_today.to_string(),
            Style::default().fg(Color::White),
        ),
        Span::raw("   "),
        Span::styled("Streak: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            state.streak.to_string(),
            Style::default().fg(Color::White),
        ),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(stats, chunks[2]);

    // Status / keybinds
    let status = Paragraph::new(state.status_line.as_str())
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(status, chunks[3]);
}