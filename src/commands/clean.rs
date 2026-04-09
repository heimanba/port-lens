//! `ports clean` — kill orphaned/zombie processes on the safe list only.

use anyhow::Result;
use dialoguer::Confirm;
use sysinfo::System;

use crate::commands::common::{self, ListenerStatus, is_safe_clean_target};

pub async fn run() -> Result<()> {
    let infos = crate::collectors::processes::collect_all()?;
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut targets: Vec<crate::collectors::processes::ProcessInfo> = infos
        .into_iter()
        .filter(|p| {
            let st = common::classify_listener(p, &sys);
            matches!(st, ListenerStatus::Zombie | ListenerStatus::Orphaned)
                && is_safe_clean_target(p)
        })
        .collect();

    targets.sort_by_key(|p| p.pid);

    if targets.is_empty() {
        println!("No orphaned or zombie processes on the safe target list.");
        return Ok(());
    }

    println!("Candidates:");
    for p in &targets {
        let st = common::classify_listener(p, &sys);
        println!("  {:>6}  {:<16}  {}", p.pid, p.name, st.as_str());
    }

    if !Confirm::new()
        .with_prompt(format!("Send SIGTERM to {} process(es)?", targets.len()))
        .default(false)
        .interact()?
    {
        return Ok(());
    }

    for p in targets {
        if let Err(e) = common::send_signal(p.pid, false) {
            eprintln!("pid {}: {e}", p.pid);
        } else {
            println!("Sent SIGTERM to {}.", p.pid);
        }
    }

    Ok(())
}
