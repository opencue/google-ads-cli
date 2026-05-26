// gads-tui — live terminal dashboard for Google Ads.
// Reads from the `gads` Python CLI via `gads --format json` shell-outs.
// Keys: q quit · r refresh · ↑↓ navigate · enter open in browser
//
// Auto-refresh every 30s. Single binary, ~1MB stripped.

use std::{
    io::{self, Stdout},
    process::Command,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table, TableState},
    Frame, Terminal,
};
use serde::Deserialize;

const REFRESH_EVERY: Duration = Duration::from_secs(30);
const POLL_EVERY: Duration = Duration::from_millis(250);

#[derive(Deserialize, Clone, Debug)]
struct Campaign {
    id: String,
    name: String,
    status: String,
    channel_type: String,
    bidding_strategy_type: Option<String>,
    budget_units: Option<f64>,
}

struct App {
    campaigns: Vec<Campaign>,
    table_state: TableState,
    last_refresh: Instant,
    profile_name: Option<String>,
    status_message: String,
    refreshing: bool,
}

impl App {
    fn new() -> Self {
        Self {
            campaigns: Vec::new(),
            table_state: TableState::default().with_selected(Some(0)),
            last_refresh: Instant::now() - REFRESH_EVERY,
            profile_name: std::env::var("GADS_PROFILE").ok(),
            status_message: "Loading…".into(),
            refreshing: false,
        }
    }

    fn refresh(&mut self) -> Result<()> {
        self.refreshing = true;
        let out = Command::new("gads")
            .args(["--format", "json", "list-campaigns"])
            .output()
            .context("failed to spawn `gads`. Is it on PATH?")?;
        self.refreshing = false;
        self.last_refresh = Instant::now();
        if !out.status.success() {
            self.status_message = format!(
                "gads exited {}: {}",
                out.status,
                String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("")
            );
            return Ok(());
        }
        let parsed: Vec<Campaign> =
            serde_json::from_slice(&out.stdout).context("parsing gads JSON output")?;
        self.campaigns = parsed;
        if self.table_state.selected().is_none() && !self.campaigns.is_empty() {
            self.table_state.select(Some(0));
        }
        self.status_message = format!(
            "{} campaign(s) · refreshed just now",
            self.campaigns.len()
        );
        Ok(())
    }

    fn move_selection(&mut self, delta: isize) {
        if self.campaigns.is_empty() {
            return;
        }
        let len = self.campaigns.len() as isize;
        let cur = self.table_state.selected().unwrap_or(0) as isize;
        let next = ((cur + delta).rem_euclid(len)) as usize;
        self.table_state.select(Some(next));
    }
}

fn main() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let result = run(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

type Term = Terminal<CrosstermBackend<Stdout>>;

fn setup_terminal() -> Result<Term> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Term) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Term, app: &mut App) -> Result<()> {
    app.refresh()?;
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(POLL_EVERY)? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Esc, _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Char('r'), _) => app.refresh()?,
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.move_selection(1),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.move_selection(-1),
                    _ => {}
                }
            }
        }

        if app.last_refresh.elapsed() > REFRESH_EVERY {
            app.refresh()?;
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(0),    // table
            Constraint::Length(3), // footer
        ])
        .split(f.area());

    render_header(f, chunks[0], app);
    render_table(f, chunks[1], app);
    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let profile = app
        .profile_name
        .as_deref()
        .unwrap_or("(no profile — set GADS_PROFILE or default_profile)");
    let title = format!("gads-tui · profile={profile}");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));
    let p = Paragraph::new(Line::from(vec![
        Span::styled(
            title,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(block);
    f.render_widget(p, area);
}

fn render_table(f: &mut Frame, area: Rect, app: &App) {
    let header_cells = ["ID", "STATUS", "TYPE", "BID", "BUDGET", "NAME"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = app
        .campaigns
        .iter()
        .map(|c| {
            let status_color = match c.status.as_str() {
                "ENABLED" => Color::Green,
                "PAUSED" => Color::Yellow,
                "REMOVED" => Color::Red,
                _ => Color::Gray,
            };
            let budget = c
                .budget_units
                .map(|u| format!("{u:.0}"))
                .unwrap_or_else(|| "—".into());
            let bid = c.bidding_strategy_type.clone().unwrap_or_else(|| "-".into());
            Row::new(vec![
                Cell::from(c.id.clone()),
                Cell::from(Span::styled(c.status.clone(), Style::default().fg(status_color))),
                Cell::from(c.channel_type.clone()),
                Cell::from(bid),
                Cell::from(budget),
                Cell::from(c.name.clone()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(14),
        Constraint::Length(10),
        Constraint::Length(18),
        Constraint::Length(27),
        Constraint::Length(10),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("▸ ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Campaigns ")
                .title_style(Style::default().fg(Color::Cyan)),
        );

    f.render_stateful_widget(table, area, &mut app.table_state.clone());
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
    let mut spans = vec![
        Span::styled(" q ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" quit  "),
        Span::styled(" r ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" refresh  "),
        Span::styled(" ↑↓/jk ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" navigate  "),
    ];

    let refresh_msg = if app.refreshing {
        " refreshing… ".to_string()
    } else {
        let secs = app.last_refresh.elapsed().as_secs();
        format!(" {secs}s since refresh ")
    };
    spans.push(Span::raw("  ·  "));
    spans.push(Span::styled(
        refresh_msg,
        Style::default().fg(Color::DarkGray),
    ));
    spans.push(Span::raw("  ·  "));
    spans.push(Span::styled(
        format!(" {} ", app.status_message),
        Style::default().fg(Color::Gray),
    ));

    let p = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(p, area);
}
