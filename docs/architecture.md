# Dragon Vault — 项目架构文档

## 1. 产品定位

Dragon Vault 是一款桌面端应用，用于**安全保存、管理和使用大模型 API Key**。基于 Tauri 2 构建，前端采用 React + TypeScript，后端采用 Rust，实现轻量、安全、跨平台的本地密钥管理工具。

## 2. 技术栈总览

| 层级 | 技术 | 版本 |
|------|------|------|
| 桌面框架 | Tauri | 2.x |
| 前端框架 | React | 19.x |
| 前端语言 | TypeScript | 6.x |
| 构建工具 | Vite | 8.x |
| 后端语言 | Rust | edition 2021 |
| 序列化 | serde + serde_json | 1.x |
| 包管理 | pnpm (前端) / cargo (后端) | — |

## 3. 目录结构

```
dragon-vault/
├── docs/                          # 项目文档（架构、规范、指南）
│   ├── architecture.md            # 本文档
│   ├── rust-guidelines.md         # Rust 编码规范
│   └── typescript-guidelines.md   # TypeScript/React 编码规范
│
├── src/                           # 前端源码（TypeScript + React）
│   ├── assets/                    # 静态资源（图片、SVG 等）
│   ├── components/                # 可复用 UI 组件
│   │   ├── common/                # 通用基础组件（Button, Input, Modal 等）
│   │   └── business/              # 业务组件（ApiKeyCard, ProviderForm 等）
│   ├── pages/                     # 页面级组件
│   │   ├── Dashboard/             # 仪表盘/总览页
│   │   ├── KeyManager/            # API Key 管理页
│   │   ├── ProviderSettings/      # 模型服务商配置页
│   │   └── Settings/              # 应用设置页
│   ├── hooks/                     # 自定义 React Hooks
│   ├── services/                  # 前端服务层（封装 Tauri invoke 调用）
│   │   ├── api.ts                 # Tauri command 调用封装
│   │   └── types.ts               # 前端业务类型定义
│   ├── stores/                    # 状态管理（Context 或轻量状态库）
│   ├── utils/                     # 工具函数
│   ├── styles/                    # 全局样式、主题变量
│   ├── App.tsx                    # 根组件
│   ├── App.css                    # 根组件样式
│   └── main.tsx                   # 应用入口
│
├── src-tauri/                     # 后端源码（Rust + Tauri）
│   ├── src/
│   │   ├── main.rs                # 二进制入口
│   │   ├── lib.rs                 # 库入口，Tauri Builder 配置
│   │   ├── commands/              # Tauri command 处理器（前端可调用的接口）
│   │   │   ├── mod.rs
│   │   │   ├── key_commands.rs    # API Key CRUD 命令
│   │   │   ├── provider_commands.rs # 服务商管理命令
│   │   │   └── app_commands.rs    # 应用级命令（设置、状态等）
│   │   ├── models/                # 数据模型定义
│   │   │   ├── mod.rs
│   │   │   ├── api_key.rs         # API Key 数据结构
│   │   │   ├── provider.rs        # 服务商数据结构
│   │   │   └── settings.rs        # 应用设置数据结构
│   │   ├── services/              # 业务逻辑层
│   │   │   ├── mod.rs
│   │   │   ├── crypto.rs          # 加密/解密服务（核心安全模块）
│   │   │   ├── key_service.rs     # API Key 业务逻辑
│   │   │   └── storage.rs         # 持久化存储抽象层
│   │   ├── storage/               # 存储实现
│   │   │   ├── mod.rs
│   │   │   └── local_storage.rs   # 本地文件加密存储实现
│   │   └── error.rs               # 统一错误类型定义
│   ├── capabilities/              # Tauri 权限声明
│   │   └── default.json
│   ├── icons/                     # 应用图标资源
│   ├── Cargo.toml                 # Rust 依赖配置
│   ├── tauri.conf.json            # Tauri 应用配置
│   └── build.rs                   # 构建脚本
│
├── public/                        # 前端公共静态资源
├── index.html                     # HTML 入口
├── package.json                   # 前端依赖配置
├── tsconfig.json                  # TypeScript 编译配置
├── tsconfig.node.json             # Node 端 TypeScript 配置
├── vite.config.ts                 # Vite 构建配置
└── .gitignore
```

## 4. 架构分层

```
┌─────────────────────────────────────────────────┐
│                   前端 (React + TS)               │
│  ┌───────────┐  ┌──────────┐  ┌──────────────┐  │
│  │   Pages   │──│ Components│  │  Hooks/Store │  │
│  └─────┬─────┘  └──────────┘  └──────┬───────┘  │
│        │                              │          │
│        └──────────┬───────────────────┘          │
│                   ▼                              │
│           services/api.ts                       │
│         (Tauri invoke 封装层)                     │
└───────────────────┬─────────────────────────────┘
                    │  IPC (Tauri Command)
┌───────────────────▼─────────────────────────────┐
│                  后端 (Rust)                      │
│  ┌──────────────────────────────────────────┐   │
│  │  commands/  — Tauri command 入口层        │   │
│  │  (参数校验、权限检查、调用 service)         │   │
│  └────────────────┬─────────────────────────┘   │
│                   ▼                              │
│  ┌──────────────────────────────────────────┐   │
│  │  services/  — 业务逻辑层                  │   │
│  │  (加密解密、密钥管理、服务商抽象)           │   │
│  └────────────────┬─────────────────────────┘   │
│                   ▼                              │
│  ┌──────────────────────────────────────────┐   │
│  │  storage/   — 持久化层                    │   │
│  │  (本地加密文件存储，可扩展其他后端)         │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

## 5. 核心模块设计

### 5.1 加密服务 (`crypto.rs`)

- 使用 AES-256-GCM 对 API Key 进行加密/解密
- 主密钥派生：基于用户主密码，使用 Argon2id 生成加密密钥
- 每次加密使用随机 Nonce，确保相同明文产生不同密文
- 内存安全：使用后立即清零敏感数据

### 5.2 API Key 管理 (`key_service.rs`)

- Key 的 CRUD 操作
- 按服务商分组管理
- Key 的标签/分类系统
- Key 的使用记录（调用次数、最后使用时间）
- Key 的复制/测试连通性

### 5.3 存储层 (`storage.rs` + `local_storage.rs`)

- 定义 `Storage` trait 抽象接口
- `LocalStorage` 实现：加密 JSON 文件存储于系统安全目录
- 数据文件路径：使用 Tauri 的 `app_data_dir`
- 支持数据导出/导入（加密备份）

### 5.4 服务商模型 (`provider.rs`)

- 预置主流大模型服务商配置（OpenAI、Anthropic、Google、国内厂商等）
- 每个服务商定义：名称、API Base URL、Key 格式校验规则、模型列表
- 支持自定义服务商

## 6. 前端页面规划

| 页面 | 路由/标识 | 功能 |
|------|-----------|------|
| Dashboard | `dashboard` | 总览：已存 Key 数量、最近使用、快速操作 |
| KeyManager | `keys` | API Key 列表、增删改查、搜索过滤、复制 |
| ProviderSettings | `providers` | 服务商管理、自定义服务商配置 |
| Settings | `settings` | 主密码管理、数据导入导出、应用偏好 |

## 7. 安全设计原则

1. **零明文存储**：所有 API Key 在磁盘上始终以密文形式存在
2. **主密码保护**：首次使用时设置主密码，用于派生加密密钥
3. **内存安全**：Rust 所有权机制 + 显式清零，最小化密钥在内存中的暴露窗口
4. **最小权限**：Tauri capability 仅声明必要权限，不开放不必要的系统访问
5. **本地优先**：所有数据存储在本地，不依赖云端同步，无网络请求泄露风险

## 8. 开发约定

- 前端通过 `services/api.ts` 统一调用 Tauri command，禁止在组件中直接 `invoke`
- Rust 端 command 层仅做参数校验和权限检查，业务逻辑下沉到 service 层
- 所有 Tauri command 返回统一的 `Result<T, AppError>` 类型
- 前端错误处理统一在 service 层捕获并转换为友好的用户提示
- 新增功能时，先定义 Rust model 和 command 接口，再开发前端对接
