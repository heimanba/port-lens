//! `ports kill` — resolve listening port → PID, or use `--pid` for raw PIDs.

use anyhow::{Context, Result};
use dialoguer::Confirm;

use crate::collectors;
use crate::collectors::ports::ListeningPort;

pub async fn run(targets: &[String], force: bool, by_pid: bool) -> Result<()> {
    let listeners = if by_pid {
        Vec::new()
    } else {
        collectors::ports::collect()?
    };

    for t in targets {
        let pid = if by_pid {
            parse_pid(t).with_context(|| format!("could not parse PID from {t:?}"))?
        } else {
            resolve_port_to_pid(&listeners, t)
                .with_context(|| format!("could not resolve port target {t:?}"))?
        };

        let sig = if force { "SIGKILL" } else { "SIGTERM" };
        if !Confirm::new()
            .with_prompt(format!("Send {sig} to PID {pid}? (from {t})"))
            .default(false)
            .interact()?
        {
            continue;
        }

        crate::commands::common::send_signal(pid, force)
            .with_context(|| format!("failed to signal {pid}"))?;
        println!("Sent {sig} to {pid}.");
    }

    Ok(())
}

fn parse_pid(s: &str) -> Result<u32> {
    let n: u32 = s
        .parse()
        .with_context(|| format!("{s:?} is not a valid process ID"))?;
    if n == 0 {
        anyhow::bail!("invalid PID 0");
    }
    Ok(n)
}

fn resolve_port_to_pid(listeners: &[ListeningPort], s: &str) -> Result<u32> {
    let port: u16 = s.parse().with_context(|| {
        format!("{s:?} is not a valid port number; use `ports kill --pid <pid>` for process IDs")
    })?;
    collectors::ports::unique_pid_for_port(listeners, port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_port_rejects_non_numeric() {
        let listeners: Vec<crate::collectors::ports::ListeningPort> = Vec::new();
        match resolve_port_to_pid(&listeners, "abc") {
            Ok(p) => panic!("expected Err, got Ok({p})"),
            Err(e) => assert!(
                format!("{e:#}").contains("not a valid port number"),
                "{e:#}"
            ),
        }
    }

    #[test]
    fn parse_pid_rejects_zero() {
        assert!(parse_pid("0").is_err());
    }

    #[test]
    fn parse_pid_accepts_typical() {
        match parse_pid("42872") {
            Ok(p) => assert_eq!(p, 42872),
            Err(e) => panic!("expected Ok(42872), got {e:#}"),
        }
    }
}
