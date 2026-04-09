# port-lens

**[English](README_en.md)** | 中文

**一款漂亮的命令行工具，帮你查看端口上正在运行什么。**

不用再猜 3000 端口被谁占用。port-lens 以彩色表格列出 TCP 监听，附带进程元数据、项目路径、框架线索和 Docker 容器名。你可以检查单个端口、安全结束监听、列出「开发向」进程（`ports ps`）、清理孤立的开发服务，以及在 TUI 中观察端口变化。

## 环境要求

- **Rust**：支持 **edition 2024** 的工具链（当前稳定版即可）。
- **操作系统**：**macOS**（主要）或 **Linux**。不支持 Windows。
- **macOS**：`lsof`（系统自带）。**Docker**：可选；当守护进程可访问时，能更好地把发布端口映射到容器。

## 快速开始

在仓库根目录执行：

```bash
cargo build --release
```

可执行文件位于 `target/release/`：

- `ports` — 主 CLI

安装到 `PATH`（示例）：

```bash
cp target/release/ports ~/.local/bin/
```

运行：

```bash
ports
ports --help
```

## 命令

| 命令 | 说明 |
|--------|-------------|
| `ports` | 列出偏开发的监听（过滤常见桌面噪声）。 |
| `ports --all` | 列出所有 TCP 监听。 |
| `ports <port>` | 检查单个端口（进程树、工作目录、git 分支、内存、可选结束提示）。 |
| `ports kill <ports…>` | 结束这些端口上的监听（`--pid` 指定 PID，`-f` / `--force` 使用 SIGKILL）。 |
| `ports ps` | 面向开发者的进程列表（CPU、内存、框架、描述）。 |
| `ports ps --all` | 与 `ps` 相同，但不限于开发类进程。 |
| `ports clean` | 仅按安全的开发运行时白名单，结束孤立/僵尸进程。 |
| `ports watch` | 监听出现或消失时的实时 TUI。 |

参数与边界情况（歧义端口、结束行为等）见 **`docs/prd.md`**。

## 架构

- **库 crate**：`port_lens`（`src/lib.rs`）— 采集器（监听、进程、Docker）、框架检测、表格与 watch UI、命令处理。
- **二进制**：`src/bin/ports.rs`（clap CLI）。
- **平台相关**的监听与工作目录解析位于共享采集抽象之后；展示与检测逻辑共用。

选用 Rust 与 CLI 形态的说明见 **[ADR-001：将 port-lens 实现为 Rust CLI](docs/decisions/ADR-001-implement-as-rust-cli.md)**。产品与行为规格见 **[docs/prd.md](docs/prd.md)**。

## 诊断

结构化日志使用 `tracing`。查看调试输出：

```bash
RUST_LOG=debug ports
```

日志输出到 **stderr**，stdout 上的表格保持干净。

## 贡献

- 格式化与静态检查：`cargo fmt` 与 `cargo clippy`（本 crate 在 `Cargo.toml` 中启用了较严格的 clippy）。
- 测试：`cargo test`。
- 重要的行为或依赖决策应在 `docs/decisions/` 下补充 ADR（可参考现有 ADR-001）。

## 许可证

MIT — 见 `Cargo.toml`（`license = "MIT"`）。

## 与 port-whisperer

本仓库是 [port-whisperer](https://github.com/LarsenCundric/port-whisperer) 的 **Rust** 实现：同样的定位（漂亮的端口/开发进程 CLI），用 Rust 重写并面向 macOS / Linux。
