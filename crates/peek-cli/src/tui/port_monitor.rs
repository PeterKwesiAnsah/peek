use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use network_inspector::tcp::{collect_all_listeners, ListenerEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortMode {
    Port,
    Process,
    Protocol,
    Pid,
}

pub struct PortMonitorApp {
    rows: Vec<ListenerEntry>,
    selected: usize,
    sort_mode: SortMode,
    should_quit: bool,
    last_error: Option<String>,
}

impl PortMonitorApp {
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            selected: 0,
            sort_mode: SortMode::Port,
            should_quit: false,
            last_error: None,
        }
    }

    fn refresh(&mut self) {
        match std::panic::catch_unwind(collect_all_listeners) {
            Ok(listeners) => {
                self.rows = listeners;
                self.apply_sort();
                if !self.rows.is_empty() {
                    self.selected = self.selected.min(self.rows.len() - 1);
                } else {
                    self.selected = 0;
                }
                self.last_error = None;
            }
            Err(_) => {
                self.last_error = Some("failed to collect listeners".to_string());
            }
        }
    }

    fn apply_sort(&mut self) {
        match self.sort_mode {
            SortMode::Port => {
                self.rows
                    .sort_by(|a, b| a.port.cmp(&b.port).then(a.protocol.cmp(&b.protocol)));
            }
            SortMode::Process => {
                self.rows.sort_by(|a, b| {
                    a.process_name
                        .cmp(&b.process_name)
                        .then(a.port.cmp(&b.port))
                });
            }
            SortMode::Protocol => {
                self.rows
                    .sort_by(|a, b| a.protocol.cmp(&b.protocol).then(a.port.cmp(&b.port)));
            }
            SortMode::Pid => {
                self.rows
                    .sort_by(|a, b| a.pid.cmp(&b.pid).then(a.port.cmp(&b.port)));
            }
        }
    }

    fn next_row(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.rows.len();
    }

    fn prev_row(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        if self.selected == 0 {
            self.selected = self.rows.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    fn cycle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Port => SortMode::Process,
            SortMode::Process => SortMode::Protocol,
            SortMode::Protocol => SortMode::Pid,
            SortMode::Pid => SortMode::Port,
        };
        self.apply_sort();
    }

    fn kill_selected(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        let row = &self.rows[self.selected];
        let pid = match row.pid {
            Some(p) if p > 0 => p,
            _ => {
                self.last_error = Some("no PID for selected socket".to_string());
                return;
            }
        };
        match kill(Pid::from_raw(pid), Signal::SIGTERM) {
            Ok(()) => {
                self.last_error = Some(format!("sent SIGTERM to pid {}", pid));
            }
            Err(e) => {
                self.last_error = Some(format!("failed to send SIGTERM to {}: {}", pid, e));
            }
        }
    }
}

pub fn run_port_monitor() -> Result<()> {
    let mut app = PortMonitorApp::new();
    app.refresh();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        event_loop(&mut terminal, &mut app)
    }));

    let restore_result = (|| -> Result<()> {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    })();

    match result {
        Ok(loop_result) => restore_result.and(loop_result),
        Err(panic) => {
            let _ = restore_result;
            std::panic::resume_unwind(panic);
        }
    }
}

fn event_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut PortMonitorApp,
) -> Result<()> {
    let mut last_tick = Instant::now();
    let interval = Duration::from_millis(3000);

    loop {
        terminal.draw(|f| draw(f, app))?;

        let poll_timeout = interval
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(100))
            .min(Duration::from_millis(200));

        if event::poll(poll_timeout)? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code, key.modifiers);
            }
        }

        if last_tick.elapsed() >= interval {
            app.refresh();
            last_tick = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_key(app: &mut PortMonitorApp, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.should_quit = true,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }
        KeyCode::Down | KeyCode::Char('j') => app.next_row(),
        KeyCode::Up | KeyCode::Char('k') => app.prev_row(),
        KeyCode::Char('s') => app.cycle_sort(),
        KeyCode::Char('r') => app.refresh(),
        KeyCode::Char('K') => app.kill_selected(),
        _ => {}
    }
}

fn draw(f: &mut Frame, app: &PortMonitorApp) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_table(f, app, chunks[1]);
    draw_footer(f, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &PortMonitorApp, area: Rect) {
    let count = app.rows.len();
    let sort_label = match app.sort_mode {
        SortMode::Port => "port",
        SortMode::Process => "process",
        SortMode::Protocol => "protocol",
        SortMode::Pid => "pid",
    };

    let mut spans = vec![
        Span::styled(
            " Port Monitor ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} sockets", count),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw("  │  "),
        Span::styled(
            format!("sort: {}", sort_label),
            Style::default().fg(Color::Magenta),
        ),
    ];

    if let Some(ref msg) = app.last_error {
        spans.push(Span::raw("  │  "));
        spans.push(Span::styled(msg.clone(), Style::default().fg(Color::Red)));
    }

    let p = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("peek --listen"),
    );
    f.render_widget(p, area);
}

fn draw_table(f: &mut Frame, app: &PortMonitorApp, area: Rect) {
    let header = Row::new(vec!["Proto", "Port", "Address", "Process", "PID"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = app
        .rows
        .iter()
        .enumerate()
        .map(|(idx, e)| {
            let proto_color = if e.protocol.starts_with("TCP") {
                Color::Green
            } else {
                Color::Cyan
            };
            let mut style = Style::default();
            if idx == app.selected {
                style = style
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }
            Row::new(vec![
                Cell::from(e.protocol.clone()).style(Style::default().fg(proto_color)),
                Cell::from(e.port.to_string()),
                Cell::from(e.address.clone()),
                Cell::from(e.process_name.clone()),
                Cell::from(
                    e.pid
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Length(6),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Length(8),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Listening sockets "),
    )
    .header(header);

    f.render_widget(table, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let spans = vec![
        Span::styled("[q] quit", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[j/k / ↑/↓] navigate", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[r] refresh", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[s] sort", Style::default().fg(Color::DarkGray)),
        Span::raw("  "),
        Span::styled("[K] SIGTERM", Style::default().fg(Color::DarkGray)),
    ];

    let p = Paragraph::new(Line::from(spans)).style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}
