<div align="center">
  <!-- TODO: 放入生成的 Logo 路径 -->
<img src="./magpie_logo.png" width="200" alt="Magpie Logo" />

  # 🐦 Project Magpie (鹊桥 Agent)

  **A High-Frequency, Dual-Engine Autonomous Travel Agent.**<br>
  *基于 Rust + Python 构建的高频出行票务监控与决策智能体*

  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
  [![Rust: 1.80+](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org/)
  [![Python: 3.12](https://img.shields.io/badge/Python-3.12-blue.svg)](https://www.python.org/)
  [![MCP Support](https://img.shields.io/badge/MCP-Ready-brightgreen.svg)](#)

</div>

---

## 💡 愿景 (Vision)
在算法与调度的交汇处，抹平信息差。

**Magpie** 不仅仅是一个抢票脚本。它是一个**7x24小时运行的自动量化买点探测器与决策引擎**。系统利用 **Rust** 极致的并发性能刺穿各种票务网关（高铁/航空），将海量脏数据固化为纯净的时序数据库（DuckDB/SQLite）；随后唤醒 **Python** 的大语言模型中枢 (LLM)，结合目标城市周末天气预报、情侣双方日程安排，自动判断“完美买点”，并静默推送到你的微信终端。

更重要的是，Magpie 正在向一个标准的 **[Model Context Protocol (MCP)](https://modelcontextprotocol.io/) Server** 演进，让你任何本地运行的 AI 助手（如 Claude Desktop / Cursor）都能瞬间拥有实时探测全网交通票务的超能力。

## 🏗 双轨架构 (Dual-Engine Architecture)
抛弃沉重的 n8n 和饱受反爬困扰的无头浏览器，我们追求 _First-Principle Thinking_ 下的性能极致：

- ⚙️ **The Core (Rust / `magpie_core`)**
  - 使用 `Tokio` 构建常驻内存的中枢网关。极低资源占用（< 10MB RAM），负责纳秒级的定时调度与数据库 (SQLite) I/O 固化。
- 🧠 **The Agent (Python / `magpie_agent`)**
  - 利用 `uv` 闪电管理环境。接受 Rust 唤起，专职对抗反爬虫体系、清洗脏数据，并利用大模型 API（DeepSeek / Qwen 等）进行复杂的意图推理与 Markdown 推送。

## 🚀 核心特性 (Features)
- [x] **极速并发穿透**：底层基于 Rust reqwest 的高频状态机，突破传统爬虫速率极限。
- [x] **时序跌落识别**：引入数据库层的滑动窗口分析，不看名义折扣，只探测历史上真正的“突然暴跌”买点。
- [ ] **多维上下文结合**：不只是“便宜”，如果目的地本周暴雨，Magpie 会告诉你：“票虽探底，但天气极差，放弃出行”。
- [ ] **🤖 Agentic MCP 接入点**：直接将我们的调度能力暴漏为 MCP 标准接口，让大模型直接读写余票与购票状态！
- [ ] **Cross-Platform**：可无缝交叉编译为 Windows `.exe` 或部署在最轻量的 Linux 软路由上。

## 📥 安装与运行 (Getting Started)
*(WIP: 当前处于内侧 Alpha 构建阶段)*

```bash
# 1. 克隆代码库
git clone https://github.com/at0rp1d0i-cell/Magpie.git
cd Magpie

# 2. 编译并启动 Rust 核心调度守护进程
cd magpie_core
cargo build --release
./target/release/magpie_core

# 3. (可选) 配置你的大模型 API Token 及微信推送
cp .env.example .env
```

## 🛣 商业化与未来演进路线 (Roadmap)
Magpie 诞生于极客情侣对抗 OTA 黑盒的痛点，但它的未来远不止于此。

1. **CLI & Bot Phase**：无需界面的微信静默助理。
2. **Native App Phase**：利用 `Tauri` 结合 Rust 底座，发布免部署、开箱即用的 Windows / MacOS 客户端。
3. **SaaS & Enterprise Phase**：云端代理所有反爬压力，中小企业通过订阅获取 “ToB 端自动化差旅比价网关” 服务。

## 🤝 贡献 (Contributing)
我们极度欢迎 Pull Request。无论你是专注于 Rust 底层并发的黑客，还是精通 Python 反侦察和 Prompt 工程的 AI 玩家，亦或是想利用这个引擎打造独立产品盈利的 Indie Hacker，Magpie 随时欢迎。

---
<div align="center">
  <i>"Don't search for flights. Let the perfect weekend find you."</i>
</div>
