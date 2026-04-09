//! `ports <port>` — detailed view and optional kill.

use anyhow::{Context, Result};
use dialoguer::Confirm;
use sysinfo::{Pid, System};

use crate::collectors;
use crate::commands::common;
use crate::detectors::framework::detect;
use crate::display::table::format_bytes_mb;

pub async fn run(port: u16) -> Result<()> {
    let listeners = collectors::ports::collect()?;
    let pid = collectors::ports::unique_pid_for_port(&listeners, port)
        .with_context(|| format!("cannot resolve a single listener for :{port}"))?;
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
