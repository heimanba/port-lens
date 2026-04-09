use anyhow::Result;

/// A single TCP port that is currently in LISTEN state.
#[derive(Debug, Clone)]
pub struct ListeningPort {
    pub port: u16,
    pub pid: u32,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Tcp6,
}

/// Collect all TCP listening ports on the current machine.
pub fn collect() -> Result<Vec<ListeningPort>> {
    #[cfg(target_os = "macos")]
    return platform::collect_macos();

    #[cfg(target_os = "linux")]
    return platform::collect_linux();

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    anyhow::bail!("port-lens is only supported on macOS and Linux");
}

/// After deduplicating rows (e.g. TCP + TCP6 for the same PID), returns the single PID
/// listening on `port`. Errors if nothing is listening or multiple distinct PIDs share the port.
pub fn unique_pid_for_port(listeners: &[ListeningPort], port: u16) -> Result<u32> {
    let mut pids: Vec<u32> = listeners
        .iter()
        .filter(|l| l.port == port)
        .map(|l| l.pid)
        .collect();
    pids.sort_unstable();
    pids.dedup();

    match pids.as_slice() {
        [] => anyhow::bail!(
            "nothing is listening on port {port}; use `ports kill --pid <pid>` if you meant a process ID"
        ),
        [pid] => Ok(*pid),
        _ => {
            let list = pids
                .iter()
                .map(u32::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!(
                "multiple processes (PIDs {list}) are listening on port {port}; use `ports kill --pid <pid>` to pick one"
            )
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use anyhow::Context;
    use std::process::Command;

    pub fn collect_macos() -> Result<Vec<ListeningPort>> {
        let which =
            which::which("lsof").context("`lsof` not found in PATH; it is required on macOS")?;

        let output = Command::new(which)
            .args(["-iTCP", "-sTCP:LISTEN", "-n", "-P", "-F", "pn"])
            .output()
            .context("failed to run lsof")?;

        parse_lsof_output(&output.stdout)
    }

    fn parse_lsof_output(raw: &[u8]) -> Result<Vec<ListeningPort>> {
        let text = std::str::from_utf8(raw).context("lsof output is not valid UTF-8")?;
        let mut ports = Vec::new();
        let mut current_pid: Option<u32> = None;

        for line in text.lines() {
            if let Some(pid_str) = line.strip_prefix('p') {
                current_pid = pid_str.parse().ok();
            } else if let Some(addr) = line.strip_prefix('n')
                && let Some(pid) = current_pid
                && let Some(port) = parse_port_from_addr(addr)
            {
                let protocol = if addr.starts_with('[') {
                    Protocol::Tcp6
                } else {
                    Protocol::Tcp
                };
                ports.push(ListeningPort {
                    port,
                    pid,
                    protocol,
                });
            }
        }

        Ok(ports)
    }

    fn parse_port_from_addr(addr: &str) -> Option<u16> {
        // formats: "*:3000", "127.0.0.1:3000", "[::1]:3000"
        addr.rsplit(':').next()?.parse().ok()
    }
}

/// Pure parsers for `ss` / `netstat` fallback on Linux (also unit-tested on other hosts).
#[cfg(any(target_os = "linux", test))]
mod linux_fallback_parse {
    use super::{ListeningPort, Protocol};

    pub fn parse_ss_output(text: &str) -> Vec<ListeningPort> {
        text.lines().filter_map(parse_ss_line).collect()
    }

    pub fn parse_netstat_output(text: &str) -> Vec<ListeningPort> {
        text.lines().filter_map(parse_netstat_line).collect()
    }

    fn parse_ss_line(line: &str) -> Option<ListeningPort> {
        if !line.contains("LISTEN") {
            return None;
        }
        let pid = extract_ss_pid(line)?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        let listen_pos = parts.iter().position(|&p| p == "LISTEN")?;
        let mut i = listen_pos + 1;
        if i < parts.len() && is_nonneg_int(parts[i]) {
            i += 1;
        }
        if i < parts.len() && is_nonneg_int(parts[i]) {
            i += 1;
        }
        let local = *parts.get(i)?;
        if local.contains("pid=") {
            return None;
        }
        let port = parse_port_from_end(local)?;
        let protocol = if local.starts_with('[') || local.contains("::") {
            Protocol::Tcp6
        } else {
            Protocol::Tcp
        };
        Some(ListeningPort {
            port,
            pid,
            protocol,
        })
    }

    fn is_nonneg_int(s: &str) -> bool {
        !s.is_empty() && s.bytes().all(|b: u8| b.is_ascii_digit())
    }

    fn extract_ss_pid(line: &str) -> Option<u32> {
        let rest = line.split("pid=").nth(1)?;
        rest.chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>()
            .parse()
            .ok()
    }

    fn parse_netstat_line(line: &str) -> Option<ListeningPort> {
        if !line.contains("LISTEN") {
            return None;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }
        let last = *parts.last()?;
        let (pid_str, _) = last.split_once('/')?;
        let pid: u32 = pid_str.parse().ok()?;
        let local = *parts.get(3)?;
        let port = parse_port_from_end(local)?;
        let protocol = if local.starts_with('[') || local.contains("::") {
            Protocol::Tcp6
        } else {
            Protocol::Tcp
        };
        Some(ListeningPort {
            port,
            pid,
            protocol,
        })
    }

    fn parse_port_from_end(s: &str) -> Option<u16> {
        s.rsplit_once(':')?.1.parse().ok()
    }

    #[cfg(test)]
    mod tests {
        use crate::collectors::ports::Protocol;

        #[test]
        fn ss_line_with_netid() {
            let line = r#"tcp   LISTEN 0      128    127.0.0.1:631      *:*    users:(("cupsd",pid=633,fd=7))"#;
            let Some(lp) = super::parse_ss_line(line) else {
                panic!("expected ss line to parse");
            };
            assert_eq!(lp.port, 631);
            assert_eq!(lp.pid, 633);
            assert!(matches!(lp.protocol, Protocol::Tcp));
        }

        #[test]
        fn ss_line_ipv6() {
            let line = r#"tcp   LISTEN 0      128    [::]:22             [::]:*    users:(("sshd",pid=1,fd=3))"#;
            let Some(lp) = super::parse_ss_line(line) else {
                panic!("expected ss line to parse");
            };
            assert_eq!(lp.port, 22);
            assert_eq!(lp.pid, 1);
            assert!(matches!(lp.protocol, Protocol::Tcp6));
        }

        #[test]
        fn ss_line_without_netid() {
            let line = r#"LISTEN 0      128    0.0.0.0:3000         0.0.0.0:*    users:(("node",pid=9999,fd=21))"#;
            let Some(lp) = super::parse_ss_line(line) else {
                panic!("expected ss line to parse");
            };
            assert_eq!((lp.port, lp.pid), (3000, 9999));
        }

        #[test]
        fn ss_skips_when_no_pid() {
            let line = "tcp   LISTEN 0      128    0.0.0.0:80         0.0.0.0:*   ";
            assert!(super::parse_ss_line(line).is_none());
        }

        #[test]
        fn netstat_line() {
            let line = "tcp        0      0 0.0.0.0:22              0.0.0.0:*               LISTEN      1234/sshd";
            let Some(lp) = super::parse_netstat_line(line) else {
                panic!("expected netstat line to parse");
            };
            assert_eq!((lp.port, lp.pid), (22, 1234));
        }

        #[test]
        fn parse_netstat_multiline() {
            let text = "Proto Recv-Q Send-Q Local Address Foreign Address State PID/Program name\n\
                        tcp        0      0 0.0.0.0:443           0.0.0.0:*               LISTEN      99/nginx\n";
            let v = super::parse_netstat_output(text);
            assert_eq!(v.len(), 1);
            assert_eq!((v[0].port, v[0].pid), (443, 99));
        }

        #[test]
        fn parse_ss_multiline() {
            let text = "Netid State Recv-Q Send-Q Local Address:Port Peer Address:PortProcess\n\
                        tcp   LISTEN 0      128    0.0.0.0:80         0.0.0.0:*    users:((\"nginx\",pid=10,fd=6))\n";
            let v = super::parse_ss_output(text);
            assert_eq!(v.len(), 1);
            assert_eq!(v[0].port, 80);
            assert_eq!(v[0].pid, 10);
        }
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::linux_fallback_parse::{parse_netstat_output, parse_ss_output};
    use super::*;
    use anyhow::Context;
    use std::fs;
    use std::process::Command;

    pub fn collect_linux() -> Result<Vec<ListeningPort>> {
        let from_proc = collect_from_proc();
        match &from_proc {
            Ok(list) if !list.is_empty() => return Ok(list.clone()),
            _ => {}
        }

        let from_cli = collect_from_ss_or_netstat()?;
        if !from_cli.is_empty() {
            return Ok(from_cli);
        }

        from_proc
    }

    fn collect_from_proc() -> Result<Vec<ListeningPort>> {
        let mut ports = Vec::new();
        ports.extend(parse_proc_net_tcp("/proc/net/tcp", Protocol::Tcp)?);
        ports.extend(parse_proc_net_tcp("/proc/net/tcp6", Protocol::Tcp6)?);
        Ok(ports)
    }

    fn collect_from_ss_or_netstat() -> Result<Vec<ListeningPort>> {
        if let Ok(cmd) = which::which("ss")
            && let Ok(out) = Command::new(cmd).args(["-tlnp"]).output()
            && out.status.success()
        {
            let text = std::str::from_utf8(&out.stdout).unwrap_or("");
            let parsed = parse_ss_output(text);
            if !parsed.is_empty() {
                return Ok(parsed);
            }
        }

        if let Ok(cmd) = which::which("netstat")
            && let Ok(out) = Command::new(cmd).args(["-tlnp"]).output()
            && out.status.success()
        {
            let text = std::str::from_utf8(&out.stdout).unwrap_or("");
            let parsed = parse_netstat_output(text);
            if !parsed.is_empty() {
                return Ok(parsed);
            }
        }

        Ok(Vec::new())
    }

    fn parse_proc_net_tcp(path: &str, protocol: Protocol) -> Result<Vec<ListeningPort>> {
        let content = fs::read_to_string(path).with_context(|| format!("failed to read {path}"))?;

        let mut ports = Vec::new();

        for line in content.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 10 {
                continue;
            }
            // state 0A = TCP_LISTEN
            if fields[3] != "0A" {
                continue;
            }
            let local_addr = fields[1];
            if let Some(port) = parse_hex_port(local_addr) {
                let inode = fields[9].parse::<u64>().unwrap_or(0);
                if let Some(pid) = inode_to_pid(inode) {
                    ports.push(ListeningPort {
                        port,
                        pid,
                        protocol: protocol.clone(),
                    });
                }
            }
        }

        Ok(ports)
    }

    fn parse_hex_port(addr: &str) -> Option<u16> {
        let hex = addr.rsplit(':').next()?;
        u16::from_str_radix(hex, 16).ok()
    }

    fn inode_to_pid(inode: u64) -> Option<u32> {
        let target = format!("socket:[{inode}]");
        for entry in fs::read_dir("/proc").ok()?.flatten() {
            let pid_str = entry.file_name();
            let pid: u32 = pid_str.to_string_lossy().parse().ok()?;
            let fd_dir = format!("/proc/{pid}/fd");
            for fd in fs::read_dir(&fd_dir).ok()?.flatten() {
                if let Ok(link) = fs::read_link(fd.path())
                    && link.to_string_lossy() == target
                {
                    return Some(pid);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod unique_pid_tests {
    use super::{ListeningPort, Protocol, unique_pid_for_port};

    fn sample() -> Vec<ListeningPort> {
        vec![
            ListeningPort {
                port: 3000,
                pid: 111,
                protocol: Protocol::Tcp,
            },
            ListeningPort {
                port: 8080,
                pid: 222,
                protocol: Protocol::Tcp6,
            },
        ]
    }

    #[test]
    fn unique_match() {
        match unique_pid_for_port(&sample(), 3000) {
            Ok(p) => assert_eq!(p, 111),
            Err(e) => panic!("expected Ok(111), got {e:#}"),
        }
    }

    #[test]
    fn dedupes_same_pid_tcp_and_tcp6() {
        let listeners = vec![
            ListeningPort {
                port: 80,
                pid: 99,
                protocol: Protocol::Tcp,
            },
            ListeningPort {
                port: 80,
                pid: 99,
                protocol: Protocol::Tcp6,
            },
        ];
        match unique_pid_for_port(&listeners, 80) {
            Ok(p) => assert_eq!(p, 99),
            Err(e) => panic!("expected Ok(99), got {e:#}"),
        }
    }

    #[test]
    fn errors_when_not_listening() {
        match unique_pid_for_port(&sample(), 9999) {
            Ok(p) => panic!("expected Err, got Ok({p})"),
            Err(e) => assert!(
                format!("{e:#}").contains("nothing is listening on port 9999"),
                "{e:#}"
            ),
        }
    }

    #[test]
    fn errors_when_ambiguous() {
        let listeners = vec![
            ListeningPort {
                port: 9000,
                pid: 1,
                protocol: Protocol::Tcp,
            },
            ListeningPort {
                port: 9000,
                pid: 2,
                protocol: Protocol::Tcp,
            },
        ];
        match unique_pid_for_port(&listeners, 9000) {
            Ok(p) => panic!("expected Err, got Ok({p})"),
            Err(e) => {
                let msg = e.to_string();
                assert!(msg.contains("multiple processes"), "{msg}");
                assert!(msg.contains("1, 2"), "{msg}");
            }
        }
    }
}
