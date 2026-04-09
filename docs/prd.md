# port-lens

**A beautiful CLI tool to see what's running on your ports.**

Stop guessing which process is hogging port 3000. `port-lens` gives you a color-coded table of every dev server, database, and background process listening on your machine -- with framework detection, Docker container identification, and interactive process management.

## What it looks like

```
$ ports

 ┌─────────────────────────────────────┐
 │  Port lens                     │
 │  listening to your ports...         │
 └─────────────────────────────────────┘

┌───────┬─────────┬───────┬──────────────────────┬────────────┬────────┬───────────┐
│ PORT  │ PROCESS │ PID   │ PROJECT              │ FRAMEWORK  │ UPTIME │ STATUS    │
├───────┼─────────┼───────┼──────────────────────┼────────────┼────────┼───────────┤
│ :3000 │ node    │ 42872 │ frontend             │ Next.js    │ 1d 9h  │ ● healthy │
├───────┼─────────┼───────┼──────────────────────┼────────────┼────────┼───────────┤
│ :3001 │ node    │ 95380 │ preview-app          │ Next.js    │ 2h 40m │ ● healthy │
├───────┼─────────┼───────┼──────────────────────┼────────────┼────────┼───────────┤
│ :4566 │ docker  │ 58351 │ backend-localstack-1 │ LocalStack │ 10d 3h │ ● healthy │
├───────┼─────────┼───────┼──────────────────────┼────────────┼────────┼───────────┤
│ :5432 │ docker  │ 58351 │ backend-postgres-1   │ PostgreSQL │ 10d 3h │ ● healthy │
├───────┼─────────┼───────┼──────────────────────┼────────────┼────────┼───────────┤
│ :6379 │ docker  │ 58351 │ backend-redis-1      │ Redis      │ 10d 3h │ ● healthy │
└───────┴─────────┴───────┴──────────────────────┴────────────┴────────┴───────────┘

  5 ports active  ·  Run ports <number> for details  ·  --all to show everything
```

Colors: green = healthy, yellow = orphaned, red = zombie.

### Status definitions

| Status | Color | Rule |
|--------|-------|------|
| **healthy** | green | Process is running normally; PPID exists and is alive |
| **orphaned** | yellow | Parent process is dead (PPID = 1 or parent no longer exists), but the process itself is still running |
| **zombie** | red | Process state is `Z` (defunct); has exited but not yet reaped by parent |

## Usage

### Show dev server ports

```bash
ports
```

Shows dev servers, Docker containers, and databases. System apps (Spotify, Raycast, etc.) are filtered out by default.

### Show all listening ports

```bash
ports --all
```

Includes system services, desktop apps, and everything else listening on your machine.

### Inspect a specific port

```bash
ports 3000
```

Detailed view: full process tree, repository path, current git branch, memory usage, and an interactive prompt to kill the process.

### Kill a process

```bash
ports kill 3000                # kill by listening port
ports kill 3000 5173 8080      # kill multiple ports
ports kill --pid 42872         # kill by process ID (avoids port vs PID ambiguity)
ports kill -f 3000             # force kill (SIGKILL)
```

Targets are **port numbers** by default: each port must have exactly one distinct listening PID (after deduplicating IPv4/IPv6). If nothing is listening, or several different processes share the port, the command fails with a hint to use `ports kill --pid <pid>`. Use `-f` when a process won't die gracefully.

### Show all dev processes

```bash
ports ps
```

A beautiful `ps aux` for developers. Shows all running dev processes (not just port-bound ones) with CPU%, memory, framework detection, and a smart description column. Docker processes are collapsed into a single summary row.

```
$ ports ps

┌───────┬─────────┬──────┬──────────┬──────────┬───────────┬─────────┬────────────────────────────────┐
│ PID   │ PROCESS │ CPU% │ MEM      │ PROJECT  │ FRAMEWORK │ UPTIME  │ WHAT                           │
├───────┼─────────┼──────┼──────────┼──────────┼───────────┼─────────┼────────────────────────────────┤
│ 592   │ Docker  │ 1.3  │ 735.5 MB │ —        │ Docker    │ 13d 12h │ 14 processes                   │
├───────┼─────────┼──────┼──────────┼──────────┼───────────┼─────────┼────────────────────────────────┤
│ 36664 │ python3 │ 0.2  │ 17.6 MB  │ —        │ Python    │ 6d 10h  │ browser_use.skill_cli.daemon   │
├───────┼─────────┼──────┼──────────┼──────────┼───────────┼─────────┼────────────────────────────────┤
│ 26408 │ node    │ 0.1  │ 9.2 MB   │ —        │ Node.js   │ 10d 13h │ jest jest_runner_cloud.js      │
├───────┼─────────┼──────┼──────────┼──────────┼───────────┼─────────┼────────────────────────────────┤
│ 25752 │ node    │ 0.0  │ 17.3 MB  │ —        │ Node.js   │ 10d 13h │ server.js                      │
├───────┼─────────┼──────┼──────────┼──────────┼───────────┼─────────┼────────────────────────────────┤
│ 66921 │ Python  │ 0.0  │ 4.1 MB   │ —        │ Python    │ 2h 25m  │ src.server                     │
└───────┴─────────┴──────┴──────────┴──────────┴───────────┴─────────┴────────────────────────────────┘

  5 processes  ·  --all to show everything
```

```bash
ports ps --all    # show all processes, not just dev
```

### Clean up orphaned processes

```bash
ports clean
```

Finds and kills orphaned or zombie dev server processes. Only targets dev runtimes -- won't touch your desktop apps.

**Safe target list** (processes `ports clean` is allowed to kill):

`node`, `python`, `python3`, `ruby`, `java`, `go`, `cargo`, `deno`, `bun`, `php`, `rails`, `uvicorn`, `gunicorn`, `puma`, `tsx`, `ts-node`, `vite`, `webpack`, `parcel`

Anything not on this list is left untouched, even if orphaned or zombie.

### Watch for port changes

```bash
ports watch
```

Real-time monitoring that notifies you whenever a port starts or stops listening.

## Platform support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS | ✅ Primary | Uses `lsof` for TCP listeners and working directory resolution |
| Linux | ✅ Secondary | Uses `/proc/net/tcp` for listeners, `/proc/{pid}/cwd` for working directory |
| Windows | ❌ Out of scope | Requires completely different APIs; not planned |

Platform-specific collection logic is abstracted behind a common `PortCollector` trait so the two supported platforms share all display and detection code.

## Framework detection

Detection runs in two passes, in priority order:

1. **File-based** (high confidence): read `package.json` `dependencies`/`devDependencies`, `requirements.txt`, `Gemfile`, `go.mod`, `Cargo.toml`, `pyproject.toml` from the process working directory.
2. **Command-line** (fallback): inspect the process argv for known patterns (e.g. `next start`, `vite`, `uvicorn`, `manage.py runserver`).

The first pass that produces a match wins. If neither matches, the FRAMEWORK column shows `—`.

**Recognized frameworks:**

- **Node.js**: Next.js, Vite, Express, Fastify, NestJS, Angular, Remix, Astro, SvelteKit, Nuxt, Webpack, Parcel, esbuild
- **Python**: Django, FastAPI, Flask, uvicorn, gunicorn, Starlette
- **Ruby**: Rails, Sinatra, Puma
- **Go**: net/http (generic), Gin, Echo, Fiber
- **Docker images**: PostgreSQL, MySQL, Redis, MongoDB, LocalStack, nginx, Elasticsearch, RabbitMQ, Kafka

## How it works

Three shell calls, runs in ~0.2s:

1. **`lsof -iTCP -sTCP:LISTEN`** -- finds all processes listening on TCP ports
2. **`ps`** (single batched call) -- retrieves process details for all PIDs at once: command line, uptime, memory, parent PID, status
3. **`lsof -d cwd`** (single batched call) -- resolves the working directory of each process to detect the project and framework

For Docker ports, a single `docker ps` call maps host ports to container names and images.

Framework detection reads `package.json` dependencies and inspects process command lines. Recognizes Next.js, Vite, Express, Angular, Remix, Astro, Django, Rails, FastAPI, and many others. Docker images are identified as PostgreSQL, Redis, MongoDB, LocalStack, nginx, etc.

The three data sources are fetched concurrently (`tokio::join!`); total wall-clock time is bounded by the slowest single call, not their sum.

## Implementation stack (Rust)

| Concern | Crate | Notes |
|---------|-------|-------|
| CLI argument parsing | `clap` v4 (derive) | subcommands, `--all`, `-f` flags |
| Static table rendering | `tabled` | custom Unicode border styles |
| Interactive TUI (`watch` mode) | `ratatui` + `ratatui-macros` | features: `scrolling-regions`, `unstable-widget-ref` |
| Terminal backend | `crossterm` | features: `event-stream` for async key events |
| Colors | `owo-colors` | RGB true color; `supports-color` for graceful fallback |
| Interactive kill prompt | `dialoguer` | y/n confirm before kill |
| Process info (CPU, memory, uptime) | `sysinfo` | cross-platform process listing |
| Unix signals (SIGTERM / SIGKILL) | `nix` | Unix-only via `[target.'cfg(unix)'.dependencies]` |
| Docker API | `bollard` | native Docker daemon client; no `docker` binary required |
| Async runtime | `tokio` | features: `rt-multi-thread`, `macros`, `process`, `signal`, `time` |
| JSON parsing (`package.json`) | `serde` + `serde_json` | framework detection |
| Application error handling | `anyhow` | |
| Library error types | `thiserror` | |
| Structured logging | `tracing` + `tracing-subscriber` | silent by default; `RUST_LOG=debug` for diagnostics |
| Unicode column width | `unicode-width` | required for correct table alignment |
| Executable detection | `which` | check `lsof` / `docker` availability before use |
| Iterator utilities | `itertools` | group_by, chain across process lists |
| Uptime formatting | (internal `fmt::uptime`) | `"1d 9h"` format; pattern from codex `utils/elapsed` |

### Binary layout

```toml
[[bin]]
name = "ports"
path = "src/bin/ports.rs"
```

### Source layout

```
src/
├── bin/
│   └── ports.rs
├── collectors/
│   ├── mod.rs
│   ├── ports.rs        # TCP listener enumeration (platform-specific)
│   ├── processes.rs    # sysinfo wrapper
│   └── docker.rs       # bollard wrapper
├── detectors/
│   ├── mod.rs
│   └── framework.rs    # two-pass detection logic
├── display/
│   ├── mod.rs
│   ├── table.rs        # static table output
│   └── watch.rs        # ratatui TUI
└── commands/
    ├── list.rs          # ports
    ├── inspect.rs       # ports <number>
    ├── kill.rs          # ports kill
    ├── ps.rs            # ports ps
    ├── clean.rs         # ports clean
    └── watch.rs         # ports watch
```
