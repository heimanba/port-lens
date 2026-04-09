//! Ratatui UI for `ports watch`.

use std::io::{Write, stdout};
use std::time::Duration;

use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

/// Run watch UI: refresh `poll` every `interval`, display table with columns.
pub fn run(
    interval: Duration,
    mut poll: impl FnMut() -> Result<Vec<Vec<String>>>,
    header: &[&str],
) -> Result<()> {
    stdout().flush().ok();
    enable_raw_mode().context("enable raw mode")?;
    let mut terminal = init_terminal().context("init terminal")?;

    let result = run_loop(&mut terminal, interval, &mut poll, header);

    restore_terminal(terminal)?;
    disable_raw_mode().context("disable raw mode")?;
    result
}

fn init_terminal() -> Result<DefaultTerminal> {
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    Ok(ratatui::init())
}

fn restore_terminal(mut terminal: DefaultTerminal) -> Result<()> {
    ratatui::restore();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).context("leave alternate screen")?;
    Ok(())
}

fn run_loop(
    terminal: &mut DefaultTerminal,
    interval: Duration,
    poll: &mut impl FnMut() -> Result<Vec<Vec<String>>>,
    header: &[&str],
) -> Result<()> {
    let mut last_refresh = std::time::Instant::now()
        .checked_sub(interval)
        .unwrap_or_else(std::time::Instant::now);
    let mut rows = poll()?;

    loop {
        if last_refresh.elapsed() >= interval {
            rows = poll()?;
            last_refresh = std::time::Instant::now();
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(1)])
                .split(f.area());

            let header_cells: Vec<Cell> = header
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().add_modifier(Modifier::BOLD)))
                .collect();
            let header_row = Row::new(header_cells).height(1);

            let table_rows: Vec<Row> = rows
                .iter()
                .map(|r| Row::new(r.iter().map(|c| Cell::from(c.as_str()))))
                .collect();

            let widths: Vec<Constraint> = header
                .iter()
                .map(|_| Constraint::Percentage((100 / header.len().max(1)) as u16))
                .collect();

            let table = Table::new(table_rows, widths).header(header_row).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" port-lens watch — q to quit "),
            );

            f.render_widget(table, chunks[0]);

            let help = ratatui::widgets::Paragraph::new("Refresh on interval · q quit")
                .style(Style::default());
            f.render_widget(help, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) if key.code == KeyCode::Char('q') => break,
                _ => {}
            }
        }
    }

    Ok(())
}
