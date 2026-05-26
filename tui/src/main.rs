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
    widgets::{Block, Borders, Cell, Clear, Padding, Paragraph, Row, Table, TableState, Wrap},
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

#[derive(Deserialize, Clone, Debug)]
struct SuggestPayload {
    optimization_score: Option<f64>,
    issues: Vec<SuggestIssue>,
}

#[derive(Deserialize, Clone, Debug)]
struct SuggestIssue {
    severity: String,
    title: String,
    detail: String,
    #[serde(default)]
    suggest: Option<String>,
}

enum InputMode {
    Budget {
        campaign_id: String,
        campaign_name: String,
        current: String,   // existing budget for display
        buffer: String,    // user-typed digits
    },
}

struct App {
    campaigns: Vec<Campaign>,
    table_state: TableState,
    last_refresh: Instant,
    profile_name: Option<String>,
    status_message: String,
    refreshing: bool,
    show_suggest: bool,
    suggest: Option<SuggestPayload>,
    input_mode: Option<InputMode>,
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
            show_suggest: false,
            suggest: None,
            input_mode: None,
        }
    }

    fn start_budget_input(&mut self) {
        if let Some(c) = self.selected_campaign() {
            self.input_mode = Some(InputMode::Budget {
                campaign_id: c.id.clone(),
                campaign_name: c.name.clone(),
                current: c.budget_units.map(|u| format!("{u:.0}")).unwrap_or("—".into()),
                buffer: String::new(),
            });
        }
    }

    fn commit_budget_input(&mut self) -> Result<()> {
        let (campaign_id, value) = if let Some(InputMode::Budget { campaign_id, buffer, .. }) = &self.input_mode {
            if buffer.is_empty() {
                self.status_message = "Budget empty — cancelled.".into();
                self.input_mode = None;
                return Ok(());
            }
            (campaign_id.clone(), buffer.clone())
        } else {
            return Ok(());
        };
        self.input_mode = None;
        self.status_message = format!("Setting budget on {campaign_id} → {value}…");
        let out = Command::new("gads")
            .env("GADS_NO_AUTOSNAPSHOT", "1")
            .args(["set-budget", &campaign_id, "--daily", &value, "--apply"])
            .output()
            .context("failed to spawn `gads set-budget`")?;
        if out.status.success() {
            self.status_message = format!("✓ budget [{campaign_id}] → {value}/day");
            self.refresh()?;
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            self.status_message = format!(
                "✗ set-budget failed: {}",
                err.lines().next().unwrap_or("(no stderr)")
            );
        }
        Ok(())
    }

    fn selected_campaign(&self) -> Option<&Campaign> {
        self.table_state.selected().and_then(|i| self.campaigns.get(i))
    }

    fn toggle_selected_status(&mut self) -> Result<()> {
        let (id, new_status) = match self.selected_campaign() {
            Some(c) => {
                let new = if c.status == "ENABLED" { "PAUSED" } else { "ENABLED" };
                (c.id.clone(), new.to_string())
            }
            None => return Ok(()),
        };
        self.status_message = format!("Setting campaign {id} → {new_status}…");
        let out = Command::new("gads")
            .env("GADS_NO_AUTOSNAPSHOT", "1") // skip per-keypress snapshot churn
            .args([
                "set-status", "campaign", &id, &new_status, "--apply",
            ])
            .output()
            .context("failed to spawn `gads set-status`")?;
        if out.status.success() {
            self.status_message = format!("✓ {id} → {new_status}");
            self.refresh()?;
        } else {
            let err = String::from_utf8_lossy(&out.stderr);
            self.status_message = format!(
                "✗ set-status failed: {}",
                err.lines().next().unwrap_or("(no stderr)")
            );
        }
        Ok(())
    }

    fn load_suggest(&mut self) -> Result<()> {
        self.status_message = "Running `gads suggest`…".into();
        let out = Command::new("gads")
            .args(["--format", "json", "suggest"])
            .output()
            .context("failed to spawn `gads suggest`")?;
        if !out.status.success() {
            self.status_message = format!(
                "✗ suggest failed: {}",
                String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("")
            );
            return Ok(());
        }
        let parsed: SuggestPayload =
            serde_json::from_slice(&out.stdout).context("parsing gads suggest JSON")?;
        let count = parsed.issues.len();
        self.suggest = Some(parsed);
        self.show_suggest = true;
        self.status_message = format!("Loaded {count} issue(s)");
        Ok(())
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

                // Budget input mode captures keys exclusively.
                if app.input_mode.is_some() {
                    match key.code {
                        KeyCode::Esc => {
                            app.input_mode = None;
                            app.status_message = "Budget input cancelled.".into();
                        }
                        KeyCode::Enter => app.commit_budget_input()?,
                        KeyCode::Backspace => {
                            if let Some(InputMode::Budget { buffer, .. }) = &mut app.input_mode {
                                buffer.pop();
                            }
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            if let Some(InputMode::Budget { buffer, .. }) = &mut app.input_mode {
                                if buffer.len() < 10 {
                                    buffer.push(c);
                                }
                            }
                        }
                        _ => {}
                    }
                    continue;
                }

                // Esc closes the suggest modal first if it's open.
                if app.show_suggest
                    && matches!(key.code, KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('q'))
                {
                    app.show_suggest = false;
                    continue;
                }
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _)
                    | (KeyCode::Esc, _)
                    | (KeyCode::Char('c'), KeyModifiers::CONTROL) => return Ok(()),
                    (KeyCode::Char('r'), _) => app.refresh()?,
                    (KeyCode::Char('p'), _) => app.toggle_selected_status()?,
                    (KeyCode::Char('b'), _) => app.start_budget_input(),
                    (KeyCode::Char('s'), _) => app.load_suggest()?,
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

    if app.show_suggest {
        render_suggest_modal(f, app);
    }
    if app.input_mode.is_some() {
        render_input_modal(f, app);
    }
}


fn render_input_modal(f: &mut Frame, app: &App) {
    let im = match &app.input_mode {
        Some(im) => im,
        None => return,
    };
    // Centered narrow modal, ~50% wide, 7 rows tall.
    let area = centered_rect(50, 25, f.area());
    f.render_widget(Clear, area);

    let (title, lines) = match im {
        InputMode::Budget { campaign_name, current, buffer, .. } => {
            let title = " Set daily budget · Enter to commit · Esc to cancel ";
            let body = vec![
                Line::from(vec![
                    Span::raw("Campaign: "),
                    Span::styled(campaign_name.clone(), Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(vec![
                    Span::raw("Current:  "),
                    Span::styled(current.clone(), Style::default().fg(Color::DarkGray)),
                    Span::raw(" /day"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("New:      "),
                    Span::styled(
                        if buffer.is_empty() { "_".to_string() } else { format!("{buffer}_") },
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" /day"),
                ]),
            ];
            (title, body)
        }
    };

    let p = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .padding(Padding::uniform(1)),
    );
    f.render_widget(p, area);
}


fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(outer[1])[1]
}


fn severity_color(sev: &str) -> Color {
    match sev {
        "P0" => Color::Red,
        "P1" => Color::Yellow,
        "P2" => Color::Cyan,
        _ => Color::Gray,
    }
}


fn render_suggest_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let payload = match &app.suggest {
        Some(p) => p,
        None => return,
    };

    let mut lines: Vec<Line> = Vec::new();
    if let Some(score) = payload.optimization_score {
        lines.push(Line::from(vec![
            Span::raw("Optimization Score: "),
            Span::styled(
                format!("{:.0}%", score * 100.0),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
    }

    if payload.issues.is_empty() {
        lines.push(Line::from(Span::styled(
            "✓ No issues flagged.",
            Style::default().fg(Color::Green),
        )));
    } else {
        for issue in payload.issues.iter().take(8) {
            let color = severity_color(&issue.severity);
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", issue.severity),
                    Style::default().bg(color).fg(Color::Black).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    issue.title.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]));
            // Detail wrapped to ~64 chars indented
            for chunk in textwrap_lines(&issue.detail, 64) {
                lines.push(Line::from(vec![
                    Span::raw("      "),
                    Span::styled(chunk, Style::default().fg(Color::Gray)),
                ]));
            }
            if let Some(s) = &issue.suggest {
                for cmd in s.lines().take(2) {
                    lines.push(Line::from(vec![
                        Span::raw("      "),
                        Span::styled(
                            format!("$ {}", cmd.trim()),
                            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                        ),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }
        if payload.issues.len() > 8 {
            lines.push(Line::from(Span::styled(
                format!("(+{} more — run `gads suggest` in the shell for full list)", payload.issues.len() - 8),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    let p = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" gads suggest · press s/Esc to close ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .padding(Padding::uniform(1)),
        );
    f.render_widget(p, area);
}


// Minimal manual word-wrap so we don't pull in a string-utils crate.
fn textwrap_lines(s: &str, max_width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        if line.is_empty() {
            line.push_str(word);
        } else if line.len() + 1 + word.len() <= max_width {
            line.push(' ');
            line.push_str(word);
        } else {
            out.push(std::mem::take(&mut line));
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
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
        Span::styled(" p ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" pause/enable  "),
        Span::styled(" b ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" budget  "),
        Span::styled(" s ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" suggest  "),
        Span::styled(" ↑↓ ", Style::default().bg(Color::DarkGray).fg(Color::White)),
        Span::raw(" nav  "),
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
