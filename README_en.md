# port-lens

**[中文](README.md)** | English

**A beautiful CLI tool to see what's running on your ports.**

Stop guessing which process is using port 3000. port-lens lists TCP listeners in a color-coded table with process metadata, project paths, framework hints, and Docker container names. You can inspect a single port, kill listeners safely, list “dev-shaped” processes (`ports ps`), clean orphaned dev servers, and watch for port changes in a TUI.

## Requirements

- **Rust**: a toolchain that supports **edition 2024** (current stable Rust).
- **OS**: **macOS** (primary) or **Linux**. Windows is not supported.
- **macOS**: `lsof` (standard). **Docker**: optional; improves mapping of published ports to containers when the daemon is reachable.

## Quick start

From the repository root:

```bash
cargo build --release
```

Binaries are written to `target/release/`:

- `ports` — main CLI

Install on your `PATH` (example):

```bash
cp target/release/ports ~/.local/bin/
```

Run:

```bash
ports
ports --help
```

Pushing a **`v*`** tag runs GitHub Actions release jobs that publish a `.tar.gz` binary archive for **Linux (Ubuntu)** and **macOS** each; there is no Windows build.

## Commands

| Command | Description |
|--------|-------------|
| `ports` | List dev-oriented listeners (filters out typical desktop noise). |
| `ports --all` | List all TCP listeners. |
| `ports <port>` | Inspect one port (process tree, cwd, git branch, memory, optional kill prompt). |
| `ports kill <ports…>` | Terminate listeners on those ports (use `--pid` for PIDs, `-f` / `--force` for SIGKILL). |
| `ports ps` | Developer-focused process list (CPU, memory, framework, description). |
| `ports ps --all` | Same as `ps` but not limited to dev processes. |
| `ports clean` | Kill orphaned/zombie processes that match a safe dev-runtime allowlist only. |
| `ports watch` | Real-time TUI when listeners appear or disappear. |

For flags and edge cases (ambiguous ports, kill behavior), see **`docs/prd.md`**.

## Architecture

- **Library crate**: `port_lens` (`src/lib.rs`) — collectors (listeners, processes, Docker), framework detection, table and watch UI, and command handlers.
- **Binaries**: `src/bin/ports.rs` (clap CLI).
- **Platform-specific** listener and cwd resolution live behind a shared collector abstraction; display and detection logic are shared.

Design rationale for choosing Rust and the CLI shape is recorded in **[ADR-001: Implement port-lens as a Rust CLI](docs/decisions/ADR-001-implement-as-rust-cli.md)**. The product and behavior spec lives in **[docs/prd.md](docs/prd.md)**.

## Diagnostics

Structured logging uses `tracing`. To see debug output:

```bash
RUST_LOG=debug ports
```

Logs go to **stderr** so tables on stdout stay clean.

## Contributing

- Format and lint: `cargo fmt` and `cargo clippy` (the crate enables strict clippy lints in `Cargo.toml`).
- Tests: `cargo test`.
- Significant behavior or dependency decisions should get an ADR under `docs/decisions/` (see existing ADR-001).

## License

MIT — see `Cargo.toml` (`license = "MIT"`).
