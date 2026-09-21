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

/** 指纹/生物解锁状态（后端含平台门控） */
export interface BiometricStatus {
  /** 设备生物硬件与系统录入是否可用 */
  available: boolean;
  /** 本应用是否已封存主密钥（可走指纹解锁） */
  enrolled: boolean;
  /** 不可用原因 */
  reason: string | null;
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

/** 余额附加行（厂商特有明细，如代金券/欠款/配额行） */
export interface BalanceExtra {
  label: string;
  value: string;
}

/** 归一化后的余额/额度信息 */
export interface BalanceInfo {
  /** 币种或额度类别（CNY / USD / 配额 等） */
  currency: string;
  total: number | null;
  granted: number | null;
  toppedUp: number | null;
  extra: BalanceExtra[];
}

/** 单模型 token 用量（仅部分厂商可提供） */
export interface ModelUsage {
  model: string;
  promptTokens: number | null;
  completionTokens: number | null;
  totalTokens: number | null;
}

/** 一次额度/用量查询的结果 */
export interface UsageReport {
  keyId: string;
  providerId: string;
  fetchedAt: string;
  balance: BalanceInfo | null;
  tokenUsage: ModelUsage[] | null;
  /** 厂商无公开接口时的说明 */
  unsupportedReason: string | null;
}

/** 持久化的用量快照（时间倒序返回） */
export interface UsageSnapshot {
  keyId: string;
  ts: string;
  totalBalance: number | null;
  totalTokens: number | null;
}
