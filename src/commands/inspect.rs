//! `ports <port>` — detailed view and optional kill.

use anyhow::{Context, Result};
use dialoguer::{Confirm, Select};
use sysinfo::{Pid, System};

use crate::collectors;
use crate::commands::common;
use crate::detectors::framework::detect;
use crate::display::table::format_bytes_mb;

pub async fn run(port: u16) -> Result<()> {
    let listeners = collectors::ports::collect()?;

    // 获取所有监听该端口的 PID
    let pids: Vec<u32> = listeners
        .iter()
        .filter(|l| l.port == port)
        .map(|l| l.pid)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if pids.is_empty() {
        anyhow::bail!("nothing is listening on port {port}");
    }

    // 如果只有一个进程，直接显示详情
    // 如果有多个进程，让用户选择
    let pid = if pids.len() == 1 {
        pids[0]
    } else {
        select_pid_from_list(port, &pids).await?
    };

    show_process_details(port, pid).await
}

/// 当多个进程监听同一端口时，显示列表让用户选择
async fn select_pid_from_list(port: u16, pids: &[u32]) -> Result<u32> {
    let (sys, pid_map) = collectors::processes::snapshot_pids(pids)?;

    println!();
    println!("Multiple processes are listening on port {port}:");
    println!();

    // 构建选择列表
    let mut options: Vec<String> = Vec::new();
    let mut pid_list: Vec<u32> = Vec::new();

    for pid in pids {
        if let Some(proc) = pid_map.get(pid) {
            let fw = detect(&proc.cmd, proc.cwd.as_deref());
            let project = common::project_label(proc.cwd.as_deref());
            let memory = format_bytes_mb(proc.memory_bytes);
            let status = common::classify_listener(proc, &sys);

            let option = format!(
                "PID {:<6} │ {:<15} │ {:<12} │ {:<10} │ {} │ {}",
                pid,
                truncate(&proc.name, 15),
                truncate(&project, 12),
                truncate(fw.display_name(), 10),
                memory,
                status.as_str()
            );
            options.push(option);
            pid_list.push(*pid);
        }
    }

    if options.is_empty() {
        anyhow::bail!("all processes disappeared");
    }

    // 使用 dialoguer 让用户选择
    let selection = Select::new()
        .with_prompt("Select a process to inspect (or press Esc to cancel)")
        .items(&options)
        .default(0)
        .interact_opt()?;

    match selection {
        Some(index) => Ok(pid_list[index]),
        None => {
            println!("\nCancelled.");
            std::process::exit(0);
        }
    }
}

/// 显示单个进程的详细信息
async fn show_process_details(port: u16, pid: u32) -> Result<()> {
    let (sys, mut pid_map) = collectors::processes::snapshot_pids(&[pid])?;
    let proc = pid_map
        .remove(&pid)
        .with_context(|| format!("process {pid} disappeared"))?;

    let fw = detect(&proc.cmd, proc.cwd.as_deref());
    let project = common::project_label(proc.cwd.as_deref());
    let uptime = common::format_uptime(&proc);
    let branch = proc.cwd.as_deref().and_then(common::git_branch);

    println!();
    println!("Port :{port}");
    println!("  PID       {pid}");
    println!("  Process   {}", proc.name);
    println!("  Project   {project}");
    println!("  Framework {}", fw.display_name());
    println!("  Uptime    {uptime}");
    println!("  Memory    {}", format_bytes_mb(proc.memory_bytes));
    if let Some(ref cwd) = proc.cwd {
        println!("  Cwd       {}", cwd.display());
    }
    if let Some(b) = branch {
        println!("  Git       {b}");
    }

    println!();
    println!("Process ancestry (child → root):");
    print_ancestry(&sys, pid);

    println!();
    if Confirm::new()
        .with_prompt("Send SIGTERM to this process?")
        .default(false)
        .interact()?
    {
        common::send_signal(pid, false).with_context(|| format!("failed to signal {pid}"))?;
        println!("Sent SIGTERM to {pid}.");
    }

    Ok(())
}

/// 截断字符串到指定长度
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn print_ancestry(sys: &System, mut pid: u32) {
    let mut seen = std::collections::HashSet::new();
    let mut line = Vec::new();
    loop {
        if !seen.insert(pid) {
            println!("  (cycle detected at pid {pid})");
            break;
        }
        let spid = Pid::from_u32(pid);
        let Some(p) = sys.process(spid) else {
            line.push(format!("{pid} (exited)"));
            break;
        };
        let name = p.name().to_string_lossy();
        line.push(format!("{pid} {name}"));
        let Some(pp) = p.parent() else {
            break;
        };
        pid = pp.as_u32();
        if pid <= 1 {
            let spid = Pid::from_u32(pid);
            if let Some(init) = sys.process(spid) {
                let n = init.name().to_string_lossy();
                line.push(format!("{pid} {n}"));
            } else {
                line.push(format!("{pid}"));
            }
            break;
        }
    }
    println!("  {}", line.join(" ← "));
}
