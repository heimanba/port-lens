use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use sysinfo::{ProcessStatus, System};

/// Snapshot of a single process.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub exe: Option<std::path::PathBuf>,
    pub cmd: Vec<String>,
    pub cwd: Option<std::path::PathBuf>,
    pub cpu_usage: f32,
    pub memory_bytes: u64,
    pub parent_pid: Option<u32>,
    pub start_time: u64,
    pub status: ProcessState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Zombie,
    Other,
}

impl From<ProcessStatus> for ProcessState {
    fn from(s: ProcessStatus) -> Self {
        match s {
            ProcessStatus::Zombie => ProcessState::Zombie,
            ProcessStatus::Run => ProcessState::Running,
            _ => ProcessState::Other,
        }
    }
}

/// Single refresh of all processes; map only requested `pids` (for parent lookup in `System`).
pub fn snapshot_pids(pids: &[u32]) -> Result<(System, HashMap<u32, ProcessInfo>)> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut map = HashMap::new();
    for &pid in pids {
        let spid = sysinfo::Pid::from_u32(pid);
        if let Some(p) = sys.process(spid) {
            map.insert(pid, to_process_info(p, pid));
        }
    }

    Ok((sys, map))
}

/// Collect process information for a specific set of PIDs.
/// More efficient than collecting all processes when only a few are needed.
pub fn collect_for_pids(pids: &[u32]) -> Result<Vec<ProcessInfo>> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let sysinfo_pids: Vec<sysinfo::Pid> = pids.iter().map(|p| sysinfo::Pid::from_u32(*p)).collect();

    let infos = sysinfo_pids
        .iter()
        .filter_map(|pid| sys.process(*pid).map(|p| to_process_info(p, pid.as_u32())))
        .collect();

    Ok(infos)
}

/// Collect all running processes.
pub fn collect_all() -> Result<Vec<ProcessInfo>> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let infos = sys
        .processes()
        .iter()
        .map(|(pid, p)| {
            let pid_u32 = pid.as_u32();
            to_process_info(p, pid_u32)
        })
        .collect();

    Ok(infos)
}

fn to_process_info(p: &sysinfo::Process, pid: u32) -> ProcessInfo {
    let cmd: Vec<String> = p
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let cwd: Option<PathBuf> = p.cwd().map(std::path::Path::to_path_buf);

    ProcessInfo {
        pid,
        name: p.name().to_string_lossy().into_owned(),
        exe: p.exe().map(std::path::Path::to_path_buf),
        cmd,
        cwd,
        cpu_usage: p.cpu_usage(),
        memory_bytes: p.memory(),
        parent_pid: p.parent().map(sysinfo::Pid::as_u32),
        start_time: p.start_time(),
        status: p.status().into(),
    }
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn to_process_info_macos(
    p: &sysinfo::Process,
    pid: u32,
    cwd_from_lsof: Option<PathBuf>,
    cmd_from_ps: Option<String>,
) -> ProcessInfo {
    let mut cmd: Vec<String> = p
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().into_owned())
        .collect();
    let mut cwd: Option<PathBuf> = p.cwd().map(std::path::Path::to_path_buf);

    // Use batch-fetched data if available
    cwd = cwd.or(cwd_from_lsof);
    if macos_cmd_needs_ps_enrichment(&cmd)
        && let Some(line) = cmd_from_ps
    {
        cmd = vec![line];
    }

    ProcessInfo {
        pid,
        name: p.name().to_string_lossy().into_owned(),
        exe: p.exe().map(std::path::Path::to_path_buf),
        cmd,
        cwd,
        cpu_usage: p.cpu_usage(),
        memory_bytes: p.memory(),
        parent_pid: p.parent().map(sysinfo::Pid::as_u32),
        start_time: p.start_time(),
        status: p.status().into(),
    }
}

/// Batch fetch cwd and cmd for all processes using a single lsof and ps command.
/// Returns (cwd_map, cmd_map) where keys are pid.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn macos_batch_fetch_cwd_and_cmd(sys: &System) -> (HashMap<u32, PathBuf>, HashMap<u32, String>) {
    let mut cwd_map = HashMap::new();
    let mut cmd_map = HashMap::new();

    // Collect all PIDs
    let pids: Vec<u32> = sys.processes().keys().map(|p| p.as_u32()).collect();
    if pids.is_empty() {
        return (cwd_map, cmd_map);
    }

    // Batch fetch cwd using lsof
    if let Ok(lsof) = which::which("lsof") {
        let pid_args: Vec<String> = pids
            .iter()
            .flat_map(|pid| ["-p".to_string(), pid.to_string()])
            .collect();
        let args: Vec<&str> = std::iter::once("-a")
            .chain(std::iter::once("-d"))
            .chain(std::iter::once("cwd"))
            .chain(pid_args.iter().map(std::string::String::as_str))
            .collect();

        if let Ok(output) = std::process::Command::new(&lsof).args(&args).output()
            && output.status.success()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 9 {
                    continue;
                }
                if let Ok(pid) = parts[1].parse::<u32>() {
                    let path = parts[8..].join(" ");
                    if path.starts_with('/') {
                        cwd_map.insert(pid, PathBuf::from(path));
                    }
                }
            }
        }
    }

    // Batch fetch cmd using ps
    let pid_list = pids
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    if let Ok(output) = std::process::Command::new("/bin/ps")
        .args(["-p", &pid_list, "-ww", "-o", "pid=,args="])
        .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Parse "pid args" format
            let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
            if let Some(pid_str) = parts.first()
                && let Ok(pid) = pid_str.trim().parse::<u32>()
            {
                let cmd = parts
                    .get(1)
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if !cmd.is_empty() {
                    cmd_map.insert(pid, cmd);
                }
            }
        }
    }

    (cwd_map, cmd_map)
}

/// On macOS, `sysinfo` often omits `cwd` and truncates `cmd` for other processes. Fill from CLI
/// tools (same idea as port-whisperer) so PROJECT / FRAMEWORK columns can resolve.
#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn macos_cwd_via_lsof(pid: u32) -> Option<PathBuf> {
    let lsof = which::which("lsof").ok()?;
    let output = std::process::Command::new(lsof)
        .args(["-a", "-d", "cwd", "-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }
        let path = parts[8..].join(" ");
        if path.starts_with('/') {
            return Some(PathBuf::from(path));
        }
    }
    None
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn macos_cmd_needs_ps_enrichment(cmd: &[String]) -> bool {
    if cmd.is_empty() {
        return true;
    }
    // sysinfo often returns only the executable path (one "word"), not the full argv.
    cmd.join(" ").split_whitespace().count() <= 1
}

#[cfg(target_os = "macos")]
#[allow(dead_code)]
fn macos_cmd_via_ps(pid: u32) -> Option<String> {
    let output = std::process::Command::new("/bin/ps")
        .args(["-p", &pid.to_string(), "-ww", "-o", "args="])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}
