# ADR-001: Implement port-lens as a Rust CLI

## Status

Accepted

## Date

2026-04-09

## Context

port-lens is a developer-facing tool to list listening ports, attribute them to processes and projects (including Docker), detect common frameworks, and support interactive workflows (inspect, kill, watch). Requirements that drive implementation shape:

- **Reliable process and port introspection** on the host, with acceptable performance when scanning many listeners.
- **Rich terminal UX**: colorized tables, optional TUI (watch mode), and scripted use from shells and automation.
- **Single deployable artifact** users can install without a language runtime (no “install Node then npm install -g …”).
- **Ecosystem fit** for system-level data: process lists, signals on Unix, and Docker API access.

The repository is structured as a Cargo package with a library and a primary CLI binary (`ports`), using clap, tabled, ratatui, tokio, sysinfo, and bollard.

## Decision

Implement port-lens as a **Rust** application: a `lib` crate shared by the `ports` CLI binary, async I/O on **Tokio** where needed, static/tabular output via **clap** + **tabled**, and watch/interactive surfaces via **ratatui** + **crossterm**.

## Alternatives Considered

### Node.js (or Bun) CLI

- **Pros**: Fast iteration for UI-heavy CLIs; large npm ecosystem for parsing and formatting.
- **Cons**: Users must install or bundle a runtime; shipping a single static binary is heavier and more fragile; lower-friction access to low-level process/signal semantics often pushes native addons or fragile shell-outs.
- **Rejected** for the primary implementation path given the “single binary, no runtime” expectation and system-level integration goals.

### Go

- **Pros**: Simple static binaries; good stdlib and mature CLIs in the ecosystem.
- **Cons**: Team and codebase direction here standardize on Rust for this tool; duplicate stacks if the rest of the org or contributors optimize for Rust patterns (error types, async crates already chosen).
- **Rejected** as the implementation language for this repository (could be revisited if the project were restarted under different constraints).

### Shell + `lsof` / `netstat` wrappers

- **Pros**: Minimal dependencies; quick prototypes.
- **Cons**: Fragmented behavior across macOS/Linux; weak structure for Docker API, framework detection, and interactive TUI; harder to test and evolve safely.
- **Rejected** as the long-term foundation; acceptable only for throwaway experiments.

## Consequences

- **Distribution**: Release builds can ship as native binaries (see `Cargo.toml` release profile: LTO, strip); no end-user Rust toolchain required after install.
- **Concurrency**: Tokio is the default async runtime for I/O-heavy paths (e.g. Docker, subprocesses, streams); contributors should follow existing patterns rather than mixing ad hoc runtimes.
- **UX split**: One-shot commands lean on clap + tabled; live/watch flows use ratatui — new features should pick the layer that matches interaction model (batch vs interactive).
- **Platform work**: Unix-specific behavior uses `nix` / `libc` behind `cfg(unix)`; Windows or broader portability needs explicit follow-up ADRs if scope expands.
- **Onboarding**: Contributors need Rust familiarity; public CLI behavior should stay documented in `docs/prd.md` (product) and, for architectural changes, in further ADRs under `docs/decisions/`.
