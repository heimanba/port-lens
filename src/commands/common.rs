//! Shared helpers for CLI commands: status, filtering, git, safe-kill names.

use std::path::Path;
use std::process::Command;

use crate::collectors::processes::{ProcessInfo, ProcessState};
use crate::project_root::{has_project_markers, resolve_project_root};
use sysinfo::System;

/// Row status label for display (matches PRD).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListenerStatus {
    Healthy,
    Orphaned,
    Zombie,
}

impl ListenerStatus {
    /// Short English label for plain-text CLI output (e.g. `ports clean`).
    pub const fn as_str(self) -> &'static str {
        match self {
            ListenerStatus::Healthy => "healthy",
            ListenerStatus::Orphaned => "orphaned",
            ListenerStatus::Zombie => "zombie",
        }
    }
}

/// Classify a listener's owning process using parent presence in `sys`.
pub fn classify_listener(proc: &ProcessInfo, sys: &System) -> ListenerStatus {
    if proc.status == ProcessState::Zombie {
        return ListenerStatus::Zombie;
    }
    match proc.parent_pid {
        None => ListenerStatus::Orphaned,
        Some(1) => ListenerStatus::Orphaned,
        Some(ppid) => {
            if sys.process(sysinfo::Pid::from_u32(ppid)).is_some() {
                ListenerStatus::Healthy
            } else {
                ListenerStatus::Orphaned
            }
        }
    }
}

/// Uptime string from `ProcessInfo::start_time` (Unix seconds when available).
pub fn format_uptime(proc: &ProcessInfo) -> String {
    let st = proc.start_time;
    if st == 0 {
        return "—".to_string();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if st > 1_000_000_000 {
        crate::fmt::uptime(now.saturating_sub(st))
    } else {
        // Very old sysinfo semantics: treat as seconds since boot (best effort).
        crate::fmt::uptime(st)
    }
}

/// `true` if this row should appear when `ports` is run without `--all`.
pub fn include_in_default_port_view(proc: &ProcessInfo, port_is_docker_mapped: bool) -> bool {
    if port_is_docker_mapped {
        return true;
    }
    is_devish_process(proc)
}

fn is_devish_process(proc: &ProcessInfo) -> bool {
    let name = proc.name.to_lowercase();
    if DEVISH_NAME_SUBSTR.iter().any(|s| name.contains(s)) {
        return true;
    }
    if let Some(exe) = proc.exe.as_ref() {
        let lossy = exe.to_string_lossy().to_lowercase();
        if DEVISH_EXE_SUBSTR.iter().any(|s| lossy.contains(s)) {
            return true;
        }
    }
    if let Some(cwd) = proc.cwd.as_ref()
        && (has_project_markers(cwd) || resolve_project_root(cwd).is_some())
    {
        return true;
    }
    false
}

const DEVISH_NAME_SUBSTR: &[&str] = &[
    "node",
    "npm",
    "npx",
    "yarn",
    "pnpm",
    "bun",
    "deno",
    "python",
    "ruby",
    "java",
    "php",
    "docker",
    "postgres",
    "mysql",
    "mongod",
    "redis",
    "nginx",
    "com.docker",
];

const DEVISH_EXE_SUBSTR: &[&str] = &[
    "docker", "node", "python", "ruby", "postgres", "mysql", "mongod", "redis",
];

/// `ports ps` default filter: developer-ish processes only.
pub fn include_in_default_ps_view(proc: &ProcessInfo) -> bool {
    is_devish_process(proc)
}

/// Executable / command name is allowed for `ports clean` (PRD safe list).
pub fn is_safe_clean_target(proc: &ProcessInfo) -> bool {
    let cmd0 = proc.cmd.first().map(String::as_str).unwrap_or("");
    let base = Path::new(cmd0)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(cmd0);
    let base_l = base.to_lowercase();
    SAFE_CLEAN_NAMES.iter().any(|n| base_l == *n)
}

const SAFE_CLEAN_NAMES: &[&str] = &[
    "node", "python", "python3", "ruby", "java", "go", "cargo", "deno", "bun", "php", "rails",
    "uvicorn", "gunicorn", "puma", "tsx", "ts-node", "vite", "webpack", "parcel",
];

/// Send SIGTERM or SIGKILL to a process (Unix only).
#[cfg(unix)]
pub fn send_signal(pid: u32, force: bool) -> anyhow::Result<()> {
    use anyhow::Context;
    use nix::sys::signal::{Signal, kill};
    use nix::unistd::Pid;
    let sig = if force {
        Signal::SIGKILL
    } else {
        Signal::SIGTERM
    };
    kill(Pid::from_raw(pid as i32), sig).context("kill")?;
    Ok(())
}

#[cfg(not(unix))]
pub fn send_signal(_pid: u32, _force: bool) -> anyhow::Result<()> {
    anyhow::bail!("signals are only supported on Unix");
}

/// Best-effort current git branch for a repo at `cwd`.
pub fn git_branch(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

/// Project column: directory basename of nearest project root (walk up from cwd), else cwd leaf.
pub fn project_label(cwd: Option<&Path>) -> String {
    let Some(cwd) = cwd else {
        return "—".to_string();
    };
    if let Some(root) = resolve_project_root(cwd) {
        return root
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| "—".to_string());
    }
    cwd.file_name()
        .and_then(|n| n.to_str())
        .map(String::from)
        .unwrap_or_else(|| "—".to_string())
}

/// True if this process looks like part of Docker Desktop / dockerd (for `ports ps` rollup).
pub fn is_docker_family_process(proc: &ProcessInfo) -> bool {
    let n = proc.name.to_lowercase();
    if n.contains("docker") {
        return true;
    }
    if let Some(exe) = proc.exe.as_ref() {
        let e = exe.to_string_lossy().to_lowercase();
        if e.contains("docker") {
            return true;
        }
    }
    false
}
