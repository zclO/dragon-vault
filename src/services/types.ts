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

/** 更新 API Key 的请求参数（缺省字段保持不变） */
export interface UpdateApiKeyRequest {
  id: string;
  name?: string;
  value?: string;
  tags?: string[];
}

/** 服务商配置 */
export interface ProviderConfig {
  id: string;
  name: string;
  baseUrl: string;
  keyFormatPattern: string | null;
  models: string[];
  isBuiltIn: boolean;
}

export type ThemeMode = "light" | "dark" | "system";

/** 保险库状态 */
export interface VaultStatus {
  initialized: boolean;
  unlocked: boolean;
}

/** 应用设置 */
export interface AppSettings {
  theme: ThemeMode;
  autoLockMinutes: number;
  language: string;
}

/** 仪表盘统计数据 */
export interface DashboardStats {
  totalKeys: number;
  totalProviders: number;
  recentUsage: number;
}
