//! TCP listener rows shared by `ports` (static table) and `ports watch` (TUI).

use anyhow::Result;

use crate::collectors;
use crate::commands::common::{self, ListenerStatus};
use crate::detectors::framework::{detect, detect_docker_image};

#[derive(Debug, Clone)]
pub struct PortListenerRow {
    pub port: u16,
    pub process_name: String,
    pub pid: u32,
    pub project: String,
    pub framework: String,
    pub uptime: String,
    pub status: ListenerStatus,
}

/// Build one row per listening port. When `all` is false, applies the default dev-oriented filter.
pub async fn load_port_listener_rows(all: bool) -> Result<Vec<PortListenerRow>> {
    let listeners = collectors::ports::collect()?;
    let containers = collectors::docker::collect().await?;

    let mut by_port: std::collections::BTreeMap<u16, u32> = std::collections::BTreeMap::new();
    for l in listeners {
        by_port.entry(l.port).or_insert(l.pid);
    }

    let mut port_to_container: std::collections::HashMap<u16, usize> =
        std::collections::HashMap::new();
    for (idx, c) in containers.iter().enumerate() {
        for &hp in &c.host_ports {
            port_to_container.entry(hp).or_insert(idx);
        }
    }

    let pids: Vec<u32> = by_port.values().copied().collect();
    let (sys, pid_map) = collectors::processes::snapshot_pids(&pids)?;

    let mut out = Vec::new();

    for (&port, &pid) in &by_port {
        let docker_idx = port_to_container.get(&port).copied();
        let is_docker_mapped = docker_idx.is_some();

        let proc = match pid_map.get(&pid) {
            Some(p) => p,
            None => continue,
        };

        if !all && !common::include_in_default_port_view(proc, is_docker_mapped) {
            continue;
        }

        let (process_name, project, framework, status) =
            if let Some(ci) = docker_idx.and_then(|i| containers.get(i)) {
                let fw = detect_docker_image(&ci.image);
                (
                    "docker".to_string(),
                    ci.name.clone(),
                    fw.display_name().to_string(),
                    ListenerStatus::Healthy,
                )
            } else {
                let fw = detect(&proc.cmd, proc.cwd.as_deref());
                let st = common::classify_listener(proc, &sys);
                (
                    proc.name.clone(),
                    common::project_label(proc.cwd.as_deref()),
                    fw.display_name().to_string(),
                    st,
                )
            };

        let uptime = common::format_uptime(proc);

        out.push(PortListenerRow {
            port,
            process_name,
            pid,
            project,
            framework,
            uptime,
            status,
        });
    }

    Ok(out)
}
