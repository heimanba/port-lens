//! Default `ports` listing (TCP listeners table).

use anyhow::Result;

use crate::commands::port_rows;
use crate::display::table;

pub async fn run(all: bool) -> Result<()> {
    let rows = port_rows::load_port_listener_rows(all).await?;
    let display: Vec<_> = rows
        .into_iter()
        .map(|r| {
            (
                r.port,
                r.process_name,
                r.pid,
                r.project,
                r.framework,
                r.uptime,
                r.status,
            )
        })
        .collect();

    table::print_port_banner();
    if display.is_empty() {
        println!("No matching listening ports.");
    } else {
        table::print_port_table(&display)?;
    }

    println!();
    println!(
        "  {} ports active  ·  Run ports <number> for details  ·  --all to show everything",
        display.len()
    );

    Ok(())
}
