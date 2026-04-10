//! Static Unicode tables for `ports` and `ports ps`.

use std::io::{self, Write};

use owo_colors::OwoColorize;
use tabled::Table;
use tabled::builder::Builder;
use tabled::settings::{Style, Width, peaker::PriorityMax};

use crate::commands::common::ListenerStatus;
use crate::locale::UiLang;

/// Get terminal width, default to 80 if unavailable
fn get_terminal_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

/// Minimum reasonable width for port table (7 columns with some content)
const MIN_PORT_TABLE_WIDTH: usize = 80;
/// Minimum reasonable width for ps table (8 columns)
const MIN_PS_TABLE_WIDTH: usize = 90;

/// Horizontal line length between `┌`/`└` and `┐`/`┘` (inner width between `│` columns).
const BANNER_INNER_WIDTH: usize = 37;
/// Two spaces after the left `│` before title/subtitle text.
const BANNER_TEXT_WIDTH: usize = BANNER_INNER_WIDTH - 2;

/// One row for `ports ps` (PID, process, CPU%, mem, project, framework, uptime, what).
pub type PsTableRow = (u32, String, String, String, String, String, String, String);

fn print_banner_row(label: &str, styled: impl std::fmt::Display) {
    let pad = BANNER_TEXT_WIDTH.saturating_sub(label.len());
    println!(" │  {}{}│", styled, " ".repeat(pad));
}

/// Render the main port table to stdout.
pub fn print_port_banner() {
    let lang = UiLang::current();
    let rule = "─".repeat(BANNER_INNER_WIDTH);
    let title = crate::locale::banner_title(lang);
    let subtitle = crate::locale::banner_subtitle(lang);
    println!();
    println!(" ┌{rule}┐");
    print_banner_row(title, title.bold());
    print_banner_row(subtitle, subtitle.dimmed());
    println!(" └{rule}┘");
    println!();
}

/// Build rows and print. Status column uses ANSI colors.
pub fn print_port_table(
    rows: &[(u16, String, u32, String, String, String, ListenerStatus)],
) -> io::Result<()> {
    let lang = UiLang::current();
    let mut builder = Builder::default();
    builder.push_record(crate::locale::port_table_headers(lang).map(String::from));
    for (port, process, pid, project, framework, uptime, status) in rows {
        builder.push_record([
            format!(":{port}"),
            process.clone(),
            pid.to_string(),
            project.clone(),
            framework.clone(),
            uptime.clone(),
            format_status_cell(*status, lang),
        ]);
    }

    let mut table: Table = builder.build();
    table.with(Style::modern_rounded());

    // Limit table width to terminal width, but ensure minimum readability
    let term_width = get_terminal_width().max(MIN_PORT_TABLE_WIDTH);
    table.with(Width::truncate(term_width).priority(PriorityMax::new(false)));

    let mut stdout = io::stdout().lock();
    write!(stdout, "{table}")?;
    writeln!(stdout)?;
    Ok(())
}

fn format_status_cell(s: ListenerStatus, lang: UiLang) -> String {
    let t = crate::locale::listener_status_label(s, lang);
    match s {
        ListenerStatus::Healthy => t.green().to_string(),
        ListenerStatus::Orphaned => t.yellow().to_string(),
        ListenerStatus::Zombie => t.red().to_string(),
    }
}

pub fn print_ps_banner() {
    println!();
}

/// `what` is a short description (argv tail or summary).
pub fn print_ps_table(rows: &[PsTableRow]) -> io::Result<()> {
    let lang = UiLang::current();
    let mut builder = Builder::default();
    builder.push_record(crate::locale::ps_table_headers(lang).map(String::from));
    for (pid, process, cpu, mem, project, framework, uptime, what) in rows {
        builder.push_record([
            pid.to_string(),
            process.clone(),
            cpu.clone(),
            mem.clone(),
            project.clone(),
            framework.clone(),
            uptime.clone(),
            what.clone(),
        ]);
    }

    let mut table: Table = builder.build();
    table.with(Style::modern_rounded());

    // Limit table width to terminal width, but ensure minimum readability
    let term_width = get_terminal_width().max(MIN_PS_TABLE_WIDTH);
    table.with(Width::truncate(term_width).priority(PriorityMax::new(false)));

    let mut stdout = io::stdout().lock();
    write!(stdout, "{table}")?;
    writeln!(stdout)?;
    Ok(())
}

pub fn format_bytes_mb(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    format!("{mb:.1} MB")
}
