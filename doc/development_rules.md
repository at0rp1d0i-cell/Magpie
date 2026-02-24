# Magpie 开发协同规范 (Development Rules)

## 1. Git 分支与提交流程
1. **主干分支 (main)**：保持处于仅随时可发布的稳定状态。
2. **开发分支 (dev)**：所有合并到 main 的前置测试分支，功能开发全部在特性分支完成并 PR 到 dev 分支。
3. **特性分支 (feat/xxx)**：针对某一个具体的任务，例如 `feat/amadeus_api_ingestion` 或 `feat/12306_monitor`。
4. **提交信息规范 (Conventional Commits)**：
   - `feat:` 新增功能
   - `fix:` 修复 Bug
   - `chore:` 构建、基础设施变动
   - `docs:` 文档修改
   - `refactor:` 代码重构（无功能改变）

## 2. 工程原则 (Engineering Principles)
- **解耦防崩**：Rust 作为调度器绝对不允许因为爬虫子进程崩溃而闪退。必须使用安全的隔离管道 (`std::process::Command` + `stdout`/`stderr`)。
- **DRY (Don't Repeat Yourself) & SOLID**：核心存储（SQLite）、Python 爬虫接口等必须做成模块化的插件（如基于大模型上下文协议 MCP 的标准化封装）。
- **极客效率与反重名 (Supercomputing Mindset)**：尽可能减少 JSON 反序列化深度。例如 12306 返回的管道符字符串，应当优先采用 `split('|')` 映射，而非重新构建复杂的 AST 数或庞大的内存切片结构。
- **First-Principle Thinking**：逆向接口时，剥除 UI 干扰，直击 TLS 握手、HTTP Header 或后端校验的核心。不执念于“突破验证码”，而是找“免验证码”的轻量子路由。

## 3. 架构准则 (Architecture Guidelines)
- **No n8n / No Heavy Schedulers**：完全摒弃笨重的可视化流节点组件。使用极致精简的 Tokio 异步进行时间颗粒度的调度控制。
- **Zero-Trust & Private Deploy**：本系统作为私人 Agent，严禁将数据外发至不受控第三方（除非作为 DeepSeek 等大模型的必要状态下发）。
- **MCP 就绪 (MCP Ready)**：未来的爬虫及工具应当被设计成（或能被无缝封装为）标准的 Model Context Protocol (MCP) Server，以允许 AI 智能体跨系统调度。
