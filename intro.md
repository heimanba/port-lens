# ports：终端里一眼看清谁占了你的端口

起服务报「端口已被占用」？`ports` 一条命令告诉你是谁、在哪个目录、什么技术栈，不用再拼 `lsof`。

一个二进制文件，不装运行时，下载即用。适用于 macOS（推荐）与 Linux。

---

## 装上试试

从 [GitHub Releases](https://github.com/heimanba/port-lens/releases) 下载对应平台的包，解压放进 `PATH`（如 `~/.local/bin/`）。有 Rust 工具链也可以 `cargo build --release` 自行编译。

macOS 依赖系统自带的 `lsof`；有 Docker 的话会自动关联容器名和发布端口，没有也不影响核心功能。

---

## 核心用法

### 扫一眼本地监听

```bash
ports
```

![](https://img.alicdn.com/imgextra/i2/O1CN01F30wi01QblHh4l8Ou_!!6000000001995-2-tps-1576-830.png)

默认过滤桌面/系统噪声，只留开发常见的监听。想看全部：

```bash
ports --all
```

![](https://img.alicdn.com/imgextra/i2/O1CN012Vnm1u1Gc5PExi50O_!!6000000000642-2-tps-1752-1776.png)

### 深挖某个端口

```bash
ports 8000
```
![](https://img.alicdn.com/imgextra/i4/O1CN017Jwquj1COZFowfCdT_!!6000000000071-2-tps-1520-336.png)

展开进程树、工作目录、Git 分支、内存等详情，还会提示是否要结束该监听。

### 干掉占用

```bash
ports kill 3000 5173
```

也支持 `--pid` 按进程号杀；`-f` 发 SIGKILL 强杀（慎用）。

### 看进程一览

```bash
ports ps
```
![](https://img.alicdn.com/imgextra/i4/O1CN01gCdIPR1rgnGHLP0y8_!!6000000005661-2-tps-2406-1214.png)

带 CPU、内存、框架线索，方便判断是不是昨天忘关的服务。`--all` 不限于常见服务类型。

### 清理孤儿进程

```bash
ports clean
```

只清内置白名单里的开发类运行时，不会误杀。

### 实时监控

```bash
ports watch
```

TUI 界面，端口增减实时刷新，适合同时起停多个服务时盯着看。

---

## 命令速查

| 想做什么 | 命令 |
|---------|------|
| 扫一眼常见监听 | `ports` |
| 看全部 TCP 监听 | `ports --all` |
| 深挖某个端口 | `ports <端口号>` |
| 关掉指定端口 | `ports kill <端口…>` |
| 进程列表 | `ports ps` / `ports ps --all` |
| 清理孤儿进程 | `ports clean` |
| 实时监控 | `ports watch` |

完整参数以 `ports --help` 为准。

---

## 为什么快

Rust 编写，编译为单个原生二进制：

- 零运行时依赖——不需要 Node.js、Python，下载放进 PATH 就能用
- release 构建启用 LTO + 符号裁剪，产物只有几 MB
- 毫秒级出结果，没有解释器冷启动开销

对比同类 Node.js 工具，不需要 `npm install -g`，不会多出一堆 `node_modules`。

---

灵感与定位上与 [port-whisperer](https://github.com/LarsenCundric/port-whisperer) 相近。源码与构建方式见 [GitHub 仓库](https://github.com/heimanba/port-lens)。以 MIT 许可证开源。
