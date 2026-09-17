# Dragon Vault

[English](README.md) | [中文](README-ZH.md)

一款用于安全保存、管理和使用大模型 API Key 的桌面端应用。基于 [Tauri](https://tauri.app/)、[React](https://react.dev/) 和 [Rust](https://www.rust-lang.org/) 构建。

## 功能特性

- 🔐 **AES-256-GCM 加密** — 所有 API Key 加密存储，零明文落盘
- 🔑 **主密码保护** — 基于 Argon2id 算法从主密码派生加密密钥
- 🏢 **多服务商支持** — 管理 OpenAI、Anthropic、Google 等多家模型的密钥
- 🏷️ **标签与分组** — 按项目、团队或用途灵活组织密钥
- 📋 **一键复制** — 复制到剪贴板后自动清除，防止泄露
- 💾 **加密备份** — 导出和导入加密的保险库备份文件
- 🖥️ **跨平台支持** — 支持 Windows、macOS 和 Linux

## 界面预览

> _即将推出_

## 安装使用

### 从源码构建

#### 环境要求

- [Node.js](https://nodejs.org/) >= 20
- [pnpm](https://pnpm.io/)
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri 系统依赖（参见 [Tauri 环境准备](https://tauri.app/start/prerequisites/)）

#### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/your-org/dragon-vault.git
cd dragon-vault

# 安装前端依赖
pnpm install

# 启动开发模式
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

## 项目文档

- [项目架构](docs/architecture.md)
- [Rust 编码规范](docs/rust-guidelines.md)
- [TypeScript 编码规范](docs/typescript-guidelines.md)

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | [Tauri 2](https://tauri.app/) |
| 前端 | React 19 + TypeScript 6 |
| 后端 | Rust (edition 2021) |
| 构建工具 | Vite 8 |
| 加密方案 | AES-256-GCM + Argon2id |

## 项目结构

```
dragon-vault/
├── src/                  # 前端源码 (React + TypeScript)
│   ├── components/       # UI 组件
│   ├── pages/            # 页面组件
│   ├── hooks/            # 自定义 React Hooks
│   ├── services/         # Tauri IPC 调用封装
│   └── stores/           # 状态管理
├── src-tauri/            # 后端源码 (Rust)
│   └── src/
│       ├── commands/     # Tauri command 入口
│       ├── models/       # 数据模型
│       ├── services/     # 业务逻辑（加密、密钥管理）
│       └── storage/      # 持久化存储
└── docs/                 # 项目文档
```

## 参与贡献

欢迎贡献代码！提交 PR 前请先阅读[项目架构文档](docs/architecture.md)和编码规范。

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m '添加某个很棒的功能'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

<p align="center">
  用 ❤️ 为 AI 开发者打造
</p>
