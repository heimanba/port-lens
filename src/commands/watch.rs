//! `ports watch` — live TCP listener table (blocking TUI on a worker thread).

use std::time::Duration;

use anyhow::Result;
use tokio::runtime::Handle;
use tokio::task::spawn_blocking;

use crate::commands::port_rows;
use crate::locale::UiLang;

pub async fn run() -> Result<()> {
    let handle = Handle::current();
    spawn_blocking(move || {
        let headers = crate::locale::port_table_headers(UiLang::current());
        crate::display::watch::run(
            Duration::from_secs(2),
            || handle.block_on(refresh_rows()),
            headers.as_slice(),
        )
    })
    .await
    .map_err(|e| anyhow::anyhow!("watch UI task join: {e}"))?
}

async fn refresh_rows() -> Result<Vec<Vec<String>>> {
    let lang = UiLang::current();
    let rows = port_rows::load_port_listener_rows(false).await?;
    let out = rows
        .into_iter()
        .map(|r| {
            vec![
                r.port.to_string(),
                r.process_name,
                r.pid.to_string(),
                r.project,
                r.framework,
                r.uptime,
                crate::locale::listener_status_label(r.status, lang).to_string(),
            ]
        })
        .collect();
    Ok(out)
}
