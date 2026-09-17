# Dragon Vault — TypeScript / React 编码规范

## 1. 总体原则

- 使用 TypeScript **strict 模式**（已在 `tsconfig.json` 中启用）
- 禁止使用 `any` 类型，必须显式声明类型
- 禁止声明未使用的局部变量和函数参数（`noUnusedLocals` + `noUnusedParameters`）
- 优先使用 `const`，禁止使用 `var`
- 代码风格遵循项目 ESLint / Prettier 配置（待引入）
- 组件使用函数式组件 + Hooks，禁止使用 Class 组件

## 2. 项目结构规范

### 2.1 目录组织

```
src/
├── assets/                    # 静态资源
├── components/
│   ├── common/                # 通用基础组件
│   │   ├── Button.tsx
│   │   ├── Input.tsx
│   │   ├── Modal.tsx
│   │   └── index.ts           # 统一导出
│   └── business/              # 业务组件
│       ├── ApiKeyCard.tsx
│       ├── ProviderForm.tsx
│       └── index.ts
├── pages/                     # 页面组件（每个页面一个目录）
│   ├── Dashboard/
│   │   ├── index.tsx          # 页面主组件
│   │   └── Dashboard.module.css
│   ├── KeyManager/
│   │   ├── index.tsx
│   │   ├── components/        # 页面私有组件
│   │   └── KeyManager.module.css
│   ├── ProviderSettings/
│   └── Settings/
├── hooks/                     # 自定义 Hooks
│   ├── useApiKeys.ts
│   ├── useProviders.ts
│   └── useVault.ts
├── services/                  # 服务层（封装 Tauri IPC）
│   ├── api.ts                 # invoke 统一封装
│   └── types.ts               # 业务类型定义
├── stores/                    # 状态管理
│   └── vault-context.tsx
├── utils/                     # 工具函数
│   ├── format.ts              # 格式化（日期、密钥掩码等）
│   └── validation.ts          # 校验函数
├── styles/                    # 全局样式
│   ├── variables.css          # CSS 变量 / 主题色
│   └── global.css
├── App.tsx
├── App.css
└── main.tsx
```

### 2.2 文件命名

| 类型 | 命名规则 | 示例 |
|------|----------|------|
| 组件文件 | `PascalCase.tsx` | `ApiKeyCard.tsx`, `Dashboard.tsx` |
| Hook 文件 | `camelCase.ts`（`use` 前缀） | `useApiKeys.ts`, `useVault.ts` |
| 服务/工具文件 | `camelCase.ts` | `api.ts`, `format.ts` |
| 样式文件 | 与组件同名 + `.module.css` | `Dashboard.module.css` |
| 类型文件 | `types.ts` | `services/types.ts` |
| 索引文件 | `index.ts` | `components/common/index.ts` |

## 3. 命名规范

### 3.1 基本规则

| 类别 | 风格 | 示例 |
|------|------|------|
| 组件 | `PascalCase` | `ApiKeyCard`, `ProviderForm` |
| 接口/类型 | `PascalCase` | `ApiKey`, `ProviderConfig` |
| 函数/变量 | `camelCase` | `apiKeys`, `handleDelete` |
| 常量 | `SCREAMING_SNAKE_CASE` 或 `camelCase` | `MAX_RETRY_COUNT` 或 `maxRetryCount` |
| 枚举值 | `PascalCase` | `ProviderStatus.Active` |
| CSS 类名 | `kebab-case`（CSS Module） | `.api-key-card`, `.provider-form` |

### 3.2 组件命名

```tsx
// ✅ 正确：文件名与组件名一致
// ApiKeyCard.tsx
export function ApiKeyCard({ apiKey }: ApiKeyCardProps) {
  return <div className={styles['api-key-card']}>...</div>;
}

// ✅ 正确：默认导出用于页面组件
// Dashboard/index.tsx
export default function Dashboard() {
  return <main>...</main>;
}

// ❌ 错误
export default function apiKeyCard() { ... }  // 组件名应为 PascalCase
```

## 4. 类型定义规范

### 4.1 业务类型（与 Rust 模型对应）

在 `services/types.ts` 中统一定义，字段名使用 `camelCase`：

```typescript
// services/types.ts

/** API Key 摘要信息（不含明文值） */
export interface ApiKeySummary {
  id: string;
  name: string;
  providerId: string;
  providerName: string;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  lastUsedAt: string | null;
}

/** 创建 API Key 的请求参数 */
export interface CreateApiKeyRequest {
  name: string;
  providerId: string;
  value: string;
  tags?: string[];
}

/** 服务商配置 */
export interface ProviderConfig {
  id: string;
  name: string;
  baseUrl: string;
  keyFormatPattern: string;
  models: string[];
  isBuiltIn: boolean;
}

/** 应用设置 */
export interface AppSettings {
  theme: 'light' | 'dark' | 'system';
  autoLockMinutes: number;
  language: string;
}
```

### 4.2 类型定义原则

- 优先使用 `interface`，仅在需要联合类型/工具类型时使用 `type`
- 所有函数参数和返回值必须有类型注解
- Props 类型以 `组件名 + Props` 命名：`ApiKeyCardProps`
- 禁止使用 `any`，必要时使用 `unknown` 并进行类型收窄

```typescript
// ✅ 正确
function formatKeyMask(key: string): string {
  return key.slice(0, 4) + '****';
}

// ❌ 错误
function formatKeyMask(key: any): any { ... }
```

## 5. 组件规范

### 5.1 组件结构

```tsx
import { useState, useEffect } from 'react';
import styles from './ApiKeyCard.module.css';

/** 组件 Props 定义 */
interface ApiKeyCardProps {
  apiKey: ApiKeySummary;
  onEdit: (id: string) => void;
  onDelete: (id: string) => void;
}

/**
 * API Key 卡片组件
 * 展示密钥摘要信息，提供编辑和删除操作
 */
export function ApiKeyCard({ apiKey, onEdit, onDelete }: ApiKeyCardProps) {
  // 1. State
  const [isRevealed, setIsRevealed] = useState(false);

  // 2. Derived state
  const maskedValue = formatKeyMask(apiKey.id);

  // 3. Handlers
  const handleCopy = async () => {
    await navigator.clipboard.writeText(apiKey.id);
  };

  // 4. Render
  return (
    <div className={styles['api-key-card']}>
      <h3>{apiKey.name}</h3>
      <span>{maskedValue}</span>
      <button onClick={() => onEdit(apiKey.id)}>编辑</button>
      <button onClick={() => onDelete(apiKey.id)}>删除</button>
    </div>
  );
}
```

### 5.2 组件原则

- 单个组件文件不超过 **200 行**，超过则拆分子组件
- Props 不超过 **7 个**，超过则考虑合并为对象
- 组件内部状态优先，只在需要跨组件共享时提升到 Context
- 副作用（`useEffect`）必须有明确的清理逻辑（如需要）
- 禁止在渲染函数中执行异步操作

### 5.3 事件处理

```tsx
// ✅ 正确：使用 handler 命名
const handleDelete = () => { ... };
const handleSubmit = (e: React.FormEvent) => { ... };
const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => { ... };

// ❌ 错误
const delete = () => { ... };        // 保留字
const onClick = () => { ... };       // 与 DOM 事件名冲突
const btn = () => { ... };           // 含义不清
```

## 6. Service 层规范

### 6.1 Tauri invoke 封装

所有与 Rust 后端的通信必须通过 `services/api.ts` 统一封装：

```typescript
// services/api.ts
import { invoke } from '@tauri-apps/api/core';
import type { ApiKeySummary, CreateApiKeyRequest, ProviderConfig } from './types';

/** API Key 相关服务 */
export const apiKeyService = {
  /** 获取所有 API Key 列表 */
  list: (): Promise<ApiKeySummary[]> =>
    invoke('list_api_keys'),

  /** 创建新的 API Key */
  create: (req: CreateApiKeyRequest): Promise<ApiKeySummary> =>
    invoke('create_api_key', { ...req }),

  /** 删除 API Key */
  delete: (id: string): Promise<void> =>
    invoke('delete_api_key', { id }),
};

/** 服务商相关服务 */
export const providerService = {
  list: (): Promise<ProviderConfig[]> =>
    invoke('list_providers'),
};
```

### 6.2 Service 层原则

- **禁止**在组件中直接调用 `invoke()`
- 每个 service 方法必须有返回类型注解
- 参数使用具名对象，与 Rust command 参数名一致
- 错误在 service 层统一捕获并转换为友好的错误信息

```typescript
// ✅ 正确：组件通过 service 调用
const keys = await apiKeyService.list();

// ❌ 错误：组件直接 invoke
const keys = await invoke('list_api_keys');
```

## 7. 状态管理规范

### 7.1 Context 使用

```tsx
// stores/vault-context.tsx
import { createContext, useContext, useState, type ReactNode } from 'react';

interface VaultState {
  isUnlocked: boolean;
  apiKeys: ApiKeySummary[];
  unlock: (password: string) => Promise<void>;
  lock: () => void;
  refreshKeys: () => Promise<void>;
}

const VaultContext = createContext<VaultState | null>(null);

export function VaultProvider({ children }: { children: ReactNode }) {
  const [isUnlocked, setIsUnlocked] = useState(false);
  const [apiKeys, setApiKeys] = useState<ApiKeySummary[]>([]);

  const unlock = async (password: string) => {
    // ...
  };

  const lock = () => {
    setIsUnlocked(false);
    setApiKeys([]);
  };

  return (
    <VaultContext.Provider value={{ isUnlocked, apiKeys, unlock, lock, refreshKeys }}>
      {children}
    </VaultContext.Provider>
  );
}

/** 自定义 Hook 访问 Vault 状态 */
export function useVault(): VaultState {
  const context = useContext(VaultContext);
  if (!context) {
    throw new Error('useVault must be used within VaultProvider');
  }
  return context;
}
```

### 7.2 状态管理原则

- 全局状态使用 Context + useReducer（复杂时）或 useState（简单时）
- 页面级状态留在页面组件内，不放入全局
- 异步数据获取放在自定义 Hook 中，不在 Context 中直接处理
- 敏感状态（如主密码）不在 Context 中持久保存，锁定后立即清除

## 8. 样式规范

### 8.1 CSS Modules

- 使用 CSS Modules（`*.module.css`）避免样式冲突
- 全局样式仅放在 `styles/global.css` 和 `styles/variables.css`
- 主题相关颜色、间距使用 CSS 变量

```css
/* variables.css */
:root {
  --color-primary: #2563eb;
  --color-danger: #dc2626;
  --color-bg: #ffffff;
  --color-text: #1f2937;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --radius-md: 8px;
}
```

```tsx
// 组件中使用
import styles from './ApiKeyCard.module.css';

<div className={styles['api-key-card']}>
  <h3 className={styles.title}>{name}</h3>
</div>
```

### 8.2 样式原则

- 禁止使用内联样式（`style={{}}`），除非动态计算
- 禁止使用 `!important`
- 类名使用语义化命名，不使用缩写
- 响应式布局使用 CSS 变量 + media query

## 9. 错误处理规范

### 9.1 统一错误处理

```typescript
// utils/error.ts

/** 将 Rust 端返回的错误转换为用户友好信息 */
export function formatError(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return '未知错误，请稍后重试';
}

/** 在组件中展示错误提示 */
export function useErrorToast() {
  return (error: unknown) => {
    const message = formatError(error);
    // 展示 toast 通知
  };
}
```

### 9.2 错误处理原则

- 所有 `async` 操作必须有 `try/catch` 或 `.catch()` 处理
- 向用户展示的错误信息必须是中文友好提示，不暴露技术细节
- 网络/IPC 错误需区分可重试和不可重试
- 开发阶段可在控制台输出详细错误，生产阶段仅展示摘要

## 10. 安全编码规范

### 10.1 前端安全清单

- [ ] 禁止在代码中硬编码 API Key 或密钥
- [ ] 禁止将 API Key 明文写入 `localStorage`、`sessionStorage` 或 Cookie
- [ ] 密钥显示时默认掩码处理（仅显示前4位 + `****`）
- [ ] 复制操作后设置剪贴板自动清除定时器（如 30 秒后清空）
- [ ] 主密码输入框禁止浏览器自动填充（`autoComplete="off"`）
- [ ] 自动锁定：用户无操作超过设定时间后自动锁定保险库

### 10.2 密钥展示

```tsx
// utils/format.ts

/** 密钥掩码：仅显示前4位 */
export function maskApiKey(key: string): string {
  if (key.length <= 4) return '****';
  return key.slice(0, 4) + '•'.repeat(Math.min(key.length - 4, 20));
}

/** 复制到剪贴板并自动清除 */
export async function copyToClipboard(text: string, clearAfterMs = 30000) {
  await navigator.clipboard.writeText(text);
  setTimeout(() => {
    navigator.clipboard.writeText('');
  }, clearAfterMs);
}
```

## 11. 性能规范

### 11.1 渲染优化

- 列表组件使用稳定的 `key`（使用数据 ID，禁止使用数组索引）
- 大列表使用虚拟滚动（如超过 100 条数据）
- 避免在 render 中创建新对象/函数作为 props（使用 `useMemo` / `useCallback`）

### 11.2 包体积

- 使用 Vite 的 tree-shaking，确保使用具名导出
- 大型依赖使用动态 `import()` 懒加载
- 禁止引入整个 lodash，使用具名导入或原生方法
