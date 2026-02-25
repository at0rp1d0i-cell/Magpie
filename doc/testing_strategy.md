# Magpie v2 测试与安全防护体系 (Testing & Security Strategy)

随着 Magpie 从 Python 脚本成功转型为使用 Tauri (Rust + React) 的桌面独立进程，我们的安全暴露面与测试复杂性发生了质的转变。为了确保 v2 能够支撑大规模的商业或极客级分发，并应对恶意的网络投毒或隐私泄露，我们在此定义项目下一阶段的**测试结构与安全防护标准**。

## 1. 测试体系拓扑 (The Testing Topology)

我们要建立一个**漏斗型的立体防线**，从最底层的数据解析到最前端的用户交互。

### 1.1 核心层 (Core Domain) - Rust 单元测试 (Unit Tests)
- **位置:** `magpie_core/src/` 或 `magpie_desktop/src-tauri/src/` 内联 `#[cfg(test)]`
- **目标:** 确保基础解析器和转换器的绝对安全。
- **强制资产:**
  - 12306 响应结构的 JSON 变异测试（应对 12306 随时可能更改的返回值字段）。
  - 大模型 Prompt 构建器的边界测试（防止生成超长无意义 Context 打穿 Token 额度）。
  - `.env` 持久化操作的竞态锁测试（模拟高并发同时读写设置）。

### 1.2 IPC 桥接层 (Inter-Process Communication) - Rust 集成测试 (Integration Tests)
- **位置:** `magpie_desktop/src-tauri/tests/`
- **目标:** 验证前端传来指令时的防御性反应。
- **强制资产:**
  - `save_app_config` 的恶名输入阻断（例如传入超长恶意字符串以耗尽主机内存）。
  - `chat_send_message` 对特殊字符或越权执行命令（Injection）的隔离测试。

### 1.3 前端表现层 (Presentation Layer) - Playwright E2E 测试
- **位置:** `magpie_desktop/tests/`
- **目标:** 以用户视角验证渲染。
- **强制资产:**
  - 登录 / 配置大模型页面的表单遮罩机制（SecretInput）验证。
  - Dashboard 图表渲染在大数据量下的性能断言。
- **自动化:** 通过 GitHub Actions 上的无界面 (Headless) Chromium 构建冒烟测试管道。

---

## 2. 系统安全性防护重点 (Security Hardening)

作为客户端应用，用户的敏感信息（如各种服务商的 API Keys）、以及应用本身的分析对抗，构成系统安全的核心。

### 2.1 鉴权与隐私保护机制 (Privacy & Secrets)
- **本地化隔绝:** 绝不在任何云端（包括我们自己的分析服务器）记录用户的 API Key、WxPusher UID 和查询参数（Persona）。所有数据**仅留存**于用户本机的 `.env` 或加密的 SQLite 数据库中。
- **内存释放 (Zeroization):** 在解析包含 Token 的敏感网络请求后，立即释放或重置内存中的副本（使用 `zeroize` Crate）。避免内存 Dump 工具提取明文密钥。
- **IPC 指令投毒免疫:** 前端调用的 Tauri `invoke` 所有参数必须实施强类型反序列化检查，不允许执行任何动态 eval 操作或未经约束的子进程拉起 (OS Command Injection)。

### 2.2 防护机制与逆向对抗 (Anti-Reversing & Anti-Debugging)
- **前端代码混淆:** 打包（Build）时启用强制 Uglify 与代码混淆，防止被轻易窃取核心 UI 交互代码。
- **脱壳防御 (Packer Security):** 虽然 Tauri 打包的可执行文件难以完全防御逆向，但应当关闭所有的 debug 符号 (`strip = true` 在 `Cargo.toml` 中配置)，并使用 `lto = true`（链接时优化）提高代码解构复杂性。
- **XSS 与渲染安全:** React 天然免疫大部分 XSS 攻击。但在渲染来自 DeepSeek 的回答内容（特别是当支持 Markdown 时），必须采用极严格的 Sanitizer，以防大模型遭遇 Prompt 注入后下发恶意 JS 载荷至用户的内嵌 WebView。

## 3. 下一步演进计划
- 建立 `tests/` 目录存放各类的 Mock 数据资产（如 12306 返回的虚假长列表 JSON），供本地断网测试。
- 引入代码审计流水线：使用类似 `cargo-audit` 定期检测 Tauri/Tokio 底层依赖生态的安全漏洞。
