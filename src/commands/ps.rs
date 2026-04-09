//! `ports ps` — developer-oriented process list.

use anyhow::Result;

use crate::collectors::processes::{self, ProcessInfo};
use crate::commands::common::{self, is_docker_family_process};
use crate::detectors::framework::detect;
use crate::display::table::{self, PsTableRow, format_bytes_mb};

pub async fn run(all: bool) -> Result<()> {
    let infos = processes::collect_all()?;

    let (docker_procs, others): (Vec<ProcessInfo>, Vec<ProcessInfo>) =
        infos.into_iter().partition(is_docker_family_process);

    let mut rows: Vec<PsTableRow> = Vec::new();

    if !docker_procs.is_empty() {
        let n = docker_procs.len();
        let cpu: f32 = docker_procs.iter().map(|p| p.cpu_usage).sum();
        let mem: u64 = docker_procs.iter().map(|p| p.memory_bytes).sum();
        let min_pid = docker_procs.iter().map(|p| p.pid).min().unwrap_or(0);
        rows.push((
            min_pid,
            "Docker".to_string(),
            format!("{cpu:.1}"),
            format_bytes_mb(mem),
            "—".to_string(),
            "Docker".to_string(),
            "—".to_string(),
            format!("{n} processes"),
        ));
    }

    let mut rest: Vec<PsTableRow> = Vec::new();
    for p in others {
        if !all && !common::include_in_default_ps_view(&p) {
            continue;
        }
        let fw = detect(&p.cmd, p.cwd.as_deref());
        let project = common::project_label(p.cwd.as_deref());
        let uptime = common::format_uptime(&p);
        let what = what_column(&p);
        rest.push((
            p.pid,
            p.name.clone(),
            format!("{:.1}", p.cpu_usage),
            format_bytes_mb(p.memory_bytes),
            project,
            fw.display_name().to_string(),
            uptime,
            what,
        ));
    }

    rest.sort_by(|a, b| a.0.cmp(&b.0));
    rows.extend(rest);

    if rows.is_empty() {
        println!("No matching processes.");
    } else {
        table::print_ps_table(&rows)?;
    }

    println!();
    println!("  {} processes  ·  --all to show everything", rows.len());

    Ok(())
}

fn what_column(p: &ProcessInfo) -> String {
    let cmd = &p.cmd;
    match cmd.len() {
        0 => "—".to_string(),
        1 => cmd[0].clone(),
        _ => {
            let i = cmd.len() - 2;
            format!("{} {}", cmd[i], cmd[i + 1])
        }
    }
}
