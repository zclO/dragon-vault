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
  theme: "light" | "dark" | "system";
  autoLockMinutes: number;
  language: string;
}

/** 仪表盘统计数据 */
export interface DashboardStats {
  totalKeys: number;
  totalProviders: number;
  recentUsage: number;
}
