# Dragon Vault

[English](README.md) | [中文](README-ZH.md)

一款用于安全保存、管理和使用大模型 API Key 的跨平台应用，同时支持桌面端与移动端。基于 [Tauri](https://tauri.app/)、[React](https://react.dev/) 和 [Rust](https://www.rust-lang.org/) 构建。

## 功能特性

- 🔐 **AES-256-GCM 加密** — 所有 API Key 加密存储，零明文落盘
- 🔑 **主密码保护** — 基于 Argon2id 派生加密密钥，采用认证解密进行常数时间校验，防侧信道
- 🔔 **指纹解锁** — Android 端将派生主密钥封存于硬件 Keystore，验证指纹即可免密解锁
- 📱 **桌面 + 移动** — 支持 Windows、macOS、Linux 与 Android（移动端底部 Tab 导航）
- 🏢 **多服务商支持** — OpenAI、Anthropic、Google、DeepSeek 等，并可添加兼容 OpenAI 格式的自定义服务商
- 📊 **额度用量查询** — 手动拉取受支持厂商的余额与 Token 用量，快照留存查看趋势
- 🔌 **连通性测试** — 一键校验密钥能否正常访问对应服务商
- 🏷️ **标签与分组** — 按项目、团队或用途灵活组织密钥
- 📋 **一键复制** — 复制到剪贴板后自动清除，防止泄露
- ⏱️ **自动锁定** — 无操作超时后自动锁定，需重新输入主密码
- 💾 **加密备份** — 导出和导入加密的保险库备份文件

## 界面预览

> _即将推出_

## 安装使用

### 从源码构建

#### 环境要求

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri 系统依赖（参见 [Tauri 环境准备](https://tauri.app/start/prerequisites/)）
- 构建安卓版：[JDK](https://adoptium.net/) 17+、Android SDK 与 NDK（参见 [Tauri 安卓指南](https://tauri.app/mobile/android/)）

#### 桌面端

```bash
# 克隆仓库
git clone https://github.com/zclO/dragon-vault.git
cd dragon-vault

# 安装前端依赖
pnpm install

# 启动开发模式
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

#### 安卓版

```bash
# 真机/模拟器调试（热更新）
adb reverse tcp:1420 tcp:1420   # 真机需先执行此命令
pnpm android:dev

# 构建签名的 release APK
# 产物路径：src-tauri/gen/android/app/build/outputs/apk/universal/release/
pnpm android:build
```

## 项目文档

- [项目架构](docs/architecture.md)
- [Rust 编码规范](docs/rust-guidelines.md)
- [TypeScript 编码规范](docs/typescript-guidelines.md)

## 技术栈

| 层级 | 技术 |
|------|------|
| 应用框架 | [Tauri 2](https://tauri.app/)（桌面 + 安卓） |
| 前端 | React 19 + TypeScript 6 |
| 后端 | Rust (edition 2021) |
| 构建工具 | Vite 8 |
| UI | Tailwind CSS 4 + Radix UI + lucide-react |
| 加密方案 | AES-256-GCM + Argon2id |
| 生物识别 | tauri-plugin-biometry（Android Keystore） |

## 项目结构

```
dragon-vault/
├── src/                  # 前端源码 (React + TypeScript)
│   ├── components/       # 基础 UI 组件 + 布局组件
│   ├── pages/            # 页面（仪表盘、密钥、服务商、用量、设置）
│   ├── hooks/            # 自定义 React Hooks（保险库状态）
│   ├── services/         # Tauri IPC 调用封装 + 类型定义
│   └── lib/              # 通用工具函数
├── src-tauri/            # 后端源码 (Rust)
│   └── src/
│       ├── commands/     # Tauri command 入口
│       ├── models/       # 数据模型
│       ├── services/     # 业务逻辑（加密、密钥、服务商、用量、保险库）
│       └── storage/      # 持久化存储（安全存储 + 本地存储）
└── docs/                 # 项目文档
```

## 参与贡献

欢迎贡献代码！提交 PR 前请先阅读[项目架构文档](docs/architecture.md)和编码规范。

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m '添加某个很棒的功能'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

## 开源协议

本项目基于 [AGPL-3.0](LICENSE) 协议开源。

<p align="center">
  用 ❤️ 为 AI 开发者打造
</p>
