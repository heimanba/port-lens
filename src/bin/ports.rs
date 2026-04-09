use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ports",
    about = "A beautiful CLI tool to see what's running on your ports"
)]
struct Cli {
    /// Show all listening ports (system services, IDEs such as Cursor/Electron apps, etc.)
    #[arg(long)]
    all: bool,

    /// Inspect a specific port number
    port: Option<u16>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Kill a process by listening port (default) or by PID (--pid)
    Kill {
        /// Listening port numbers (must have an active listener)
        #[arg(required = true)]
        targets: Vec<String>,

        /// Treat targets as process IDs instead of port numbers
        #[arg(long)]
        pid: bool,

        /// Force kill with SIGKILL
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Show all dev processes (like ps aux for developers)
    Ps {
        /// Show all processes, not just dev
        #[arg(long)]
        all: bool,
    },

    /// Clean up orphaned and zombie dev server processes
    Clean,

    /// Watch for port changes in real time
    Watch,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(Command::Kill {
            targets,
            pid,
            force,
        }) => port_lens::commands::kill::run(&targets, force, pid).await,
        Some(Command::Ps { all }) => port_lens::commands::ps::run(all).await,
        Some(Command::Clean) => port_lens::commands::clean::run().await,
        Some(Command::Watch) => port_lens::commands::watch::run().await,
        None => {
            if let Some(port) = cli.port {
                port_lens::commands::inspect::run(port).await
            } else {
                port_lens::commands::list::run(cli.all).await
            }
        }
    }
}
