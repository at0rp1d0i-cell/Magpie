# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概览

**Magpie（鹊桥 Agent）** 是一个双引擎自动化出行票务监控智能体。系统持续轮询中国高铁（12306）和航班（VariFlight）接口，将数据固化至 SQLite，并通过 DeepSeek 大模型判断是否触发微信推送提醒。

## 仓库结构

```
Magpie/
├── magpie_core/        # 独立 Rust 守护进程（遗留/参考用）
├── magpie_desktop/     # 主体：Tauri v2 桌面客户端
│   ├── src/            # React + TypeScript 前端
│   └── src-tauri/      # Rust Tauri 后端
│       └── src/
│           ├── commands.rs   # Tauri IPC 命令（对话、配置、AI）
│           ├── daemon.rs     # 后台轮询主循环
│           ├── db.rs         # SQLite 表结构与写入
│           ├── decision.rs   # 基于 LLM 的决策引擎
│           ├── fetchers/     # train.rs（12306）+ flight.rs（VariFlight）
│           ├── models.rs     # OmniTicket、UserConfig、StationInfo
│           └── queries.rs    # 读取 DB 的 Tauri 命令
├── data/               # 运行时数据（gitignore）
│   ├── tickets.db      # SQLite 时序票务数据库
│   └── user_config.json # AI 对话生成的监控配置
└── .env                # API 密钥（gitignore）
```

## 构建与运行

所有桌面端开发均在 `magpie_desktop/` 下进行：

```bash
cd magpie_desktop

# 安装前端依赖
npm install

# 开发模式启动（同时启动 Vite + Tauri）
npm run tauri dev

# 生产构建
npm run tauri build

# 仅启动前端（不含 Rust / Tauri IPC）
npm run dev
```

独立 Rust 核心（遗留）：
```bash
cd magpie_core
cargo build --release
./target/release/magpie_core
```

## 测试

Playwright E2E 测试（在 `magpie_desktop/` 下执行）：

```bash
cd magpie_desktop

# 运行全部测试（需要开发服务器在 :1420 运行）
npx playwright test

# 运行单个测试文件
npx playwright test tests/settings.spec.ts

# 带 UI 的交互模式
npx playwright test --ui
```

Playwright 配置会在测试前自动通过 `npm run dev` 启动开发服务器（`http://localhost:1420`）。

## 配置

在项目根目录（非 `magpie_desktop/` 内）创建 `.env` 文件：

```
DEEPSEEK_API_KEY=...
DEEPSEEK_BASE_URL=https://api.deepseek.com
DEEPSEEK_MODEL=deepseek-chat
VARIFLIGHT_API_KEY=...
PUSHPLUS_TOKEN=...
WXPUSHER_UID=...
```

`data/user_config.json` 由 AI 对话流程自动生成，定义监控参数：

```json
{
  "persona": "leisure",
  "time_window_start": "2026-03-01",
  "time_window_end": "2026-03-05",
  "departure": {"city": "杭州", "train_code": "HZH", "flight_code": "HGH"},
  "destinations": [{"city": "北京", "train_code": "BJP", "flight_code": "BJS"}],
  "budget_cap": 1000
}
```

## 架构：核心数据流

### 1. AI 对话 → 配置生成 → 守护进程
- `ChatPage` 通过 Tauri IPC 调用 `chat_send_message`
- `commands.rs` 维护内存中的 `ChatState`（`Mutex<ChatState>`，含对话历史）
- 当 AI 回复中包含 ` ```json ` 代码块时，自动提取并写入 `data/user_config.json`
- 后台守护进程 `daemon.rs` 在每次轮询循环开始时读取该文件

### 2. 守护进程轮询循环
- 在 Tauri `setup()` 中通过 `tauri::async_runtime::spawn` 自动启动
- `persona == "business"` → 每 60 秒轮询；`"leisure"` → 每 3 小时轮询
- 可通过 `trigger_fetch_cycle` IPC 命令（调用 `Arc<Notify>::notify_one()`）立即唤醒
- 同时调用 `query_12306` 与 `query_variflight`，过滤 `booking_status == "Y"` 后写入 SQLite

### 3. Tauri IPC 层
所有 Rust↔前端通信通过 `lib.rs` 中注册的 Tauri 命令进行。主要命令：
- `chat_send_message` / `get_chat_history` / `clear_chat_history`
- `get_app_config` / `save_app_config`（读写 `.env` 文件）
- `trigger_fetch_cycle`（立即唤醒守护进程）
- `get_latest_tickets` / `get_daemon_status`（数据库查询）
- `get_user_plan`（读取 `data/user_config.json`）

### 4. 前端路由
React Router v7 单页应用，布局为 `Sidebar` + 主内容区。

路由：`/`（Chat）、`/dashboard`、`/plans`、`/alerts`、`/settings`

## 重要说明

- **路径解析**：Rust 后端通过 `current_dir()` 定位 `.env` 和 `data/` 目录。Tauri 开发模式下 `current_dir()` 为 `src-tauri/`，代码会向上两级定位项目根目录。
- **统一票务模型**：`OmniTicket` 是高铁与航班的统一数据结构，两个 fetcher 均将原始接口数据归一化为该结构后再写入数据库。
- **对话历史仅在内存中**：`ChatState` 由 Tauri 的 `manage()` 持有，应用重启后对话历史会丢失。
- **SQLite 版本差异**：`magpie_desktop/src-tauri/Cargo.toml` 使用 `rusqlite = { version = "0.32", features = ["bundled"] }`，`magpie_core` 使用 `0.38` 且无 bundled feature，修改时注意保持版本一致。
