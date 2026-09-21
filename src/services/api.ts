import { invoke } from "@tauri-apps/api/core";
import type {
  ApiKeySummary,
  AppSettings,
  BiometricStatus,
  CreateApiKeyRequest,
  DashboardStats,
  ProviderConfig,
  UpdateApiKeyRequest,
  VaultStatus,
} from "./types";
import { mockApiKeys, mockProviders } from "./mock-data";

/** 是否运行在 Tauri 桌面环境（纯浏览器预览时回退到 Mock 实现） */
const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** 统一调用后端 command，并把 AppError（序列化为字符串）转换为 Error */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (e) {
    throw new Error(typeof e === "string" ? e : `调用后端命令 ${command} 失败`);
  }
}

/** 模拟网络延迟 */
function delay(ms = 200): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// ---------- 浏览器预览用的内存 Mock 状态 ----------
let mockKeys: ApiKeySummary[] = [...mockApiKeys];
let mockProvidersState: ProviderConfig[] = [...mockProviders];
let mockVault: VaultStatus = { initialized: false, unlocked: false };

function mockSummary(req: CreateApiKeyRequest): ApiKeySummary {
  const now = new Date().toISOString();
  return {
    id: `key-${Date.now()}`,
    name: req.name,
    providerId: req.providerId,
    providerName: mockProvidersState.find((p) => p.id === req.providerId)?.name ?? req.providerId,
    tags: req.tags ?? [],
    createdAt: now,
    updatedAt: now,
    lastUsedAt: null,
  };
}

// ---------- 保险库 ----------

/** 保险库生命周期服务（主密码初始化 / 解锁 / 锁定） */
export const vaultService = {
  async getStatus(): Promise<VaultStatus> {
    if (!isTauri) return mockVault;
    return call<VaultStatus>("get_vault_status");
  },

  /** 首次初始化并设置主密码 */
  async initialize(password: string): Promise<void> {
    if (!isTauri) {
      await delay();
      mockVault = { initialized: true, unlocked: true };
      return;
    }
    await call("initialize_vault", { password });
  },

  /** 主密码解锁 */
  async unlock(password: string): Promise<void> {
    if (!isTauri) {
      await delay();
      mockVault = { ...mockVault, unlocked: true };
      return;
    }
    await call("unlock_vault", { password });
  },

  /** 锁定保险库并清除内存密钥 */
  async lock(): Promise<void> {
    if (!isTauri) {
      mockVault = { ...mockVault, unlocked: false };
      return;
    }
    await call("lock_vault");
  },

  /** 指纹解锁（仅移动端已封存密钥时可用） */
  async unlockWithBiometric(): Promise<void> {
    if (!isTauri) throw new Error("浏览器预览不支持指纹解锁");
    await call("unlock_with_biometric");
  },
};

// ---------- 生物解锁 ----------

/** 指纹/生物解锁服务（Windows 实现审计未通过，后端门控暂仅安卓开放） */
export const biometricService = {
  async getStatus(): Promise<BiometricStatus> {
    if (!isTauri) {
      return { available: false, enrolled: false, reason: "浏览器预览不支持指纹解锁" };
    }
    return call<BiometricStatus>("biometric_status");
  },

  /** 封存主密钥，开启指纹解锁（需已解锁） */
  async enable(): Promise<void> {
    if (!isTauri) throw new Error("浏览器预览不支持指纹解锁");
    await call("enable_biometric");
  },

  /** 移除封存密钥，关闭指纹解锁 */
  async disable(): Promise<void> {
    if (!isTauri) throw new Error("浏览器预览不支持指纹解锁");
    await call("disable_biometric");
  },
};

// ---------- API Key ----------

/** API Key 相关服务 */
export const apiKeyService = {
  /** 获取所有 API Key 列表（不含明文值） */
  async list(): Promise<ApiKeySummary[]> {
    if (!isTauri) {
      await delay();
      return [...mockKeys];
    }
    return call<ApiKeySummary[]>("list_api_keys");
  },

  /** 创建新的 API Key（值将加密存储） */
  async create(req: CreateApiKeyRequest): Promise<ApiKeySummary> {
    if (!isTauri) {
      await delay();
      const created = mockSummary(req);
      mockKeys = [...mockKeys, created];
      return created;
    }
    return call<ApiKeySummary>("create_api_key", { request: req });
  },

  /** 更新 API Key 的名称 / 值 / 标签 */
  async update(req: UpdateApiKeyRequest): Promise<ApiKeySummary> {
    if (!isTauri) {
      await delay();
      const idx = mockKeys.findIndex((k) => k.id === req.id);
      if (idx < 0) throw new Error(`API Key 不存在: ${req.id}`);
      const updated: ApiKeySummary = {
        ...mockKeys[idx],
        name: req.name?.trim() || mockKeys[idx].name,
        tags: req.tags ?? mockKeys[idx].tags,
        updatedAt: new Date().toISOString(),
      };
      mockKeys = mockKeys.map((k, i) => (i === idx ? updated : k));
      return updated;
    }
    return call<ApiKeySummary>("update_api_key", { request: req });
  },

  /** 删除 API Key */
  async delete(id: string): Promise<void> {
    if (!isTauri) {
      await delay();
      mockKeys = mockKeys.filter((k) => k.id !== id);
      return;
    }
    await call("delete_api_key", { id });
  },

  /** 解密并返回 Key 明文值（用于复制，后端会记录使用时间） */
  async reveal(id: string): Promise<string> {
    if (!isTauri) {
      await delay();
      return `sk-mock-${id.slice(-6)}`;
    }
    return call<string>("reveal_api_key", { id });
  },

  /** 测试 API Key 连通性 */
  async testConnection(id: string): Promise<boolean> {
    if (!isTauri) {
      await delay();
      return true;
    }
    return call<boolean>("test_key_connection", { id });
  },
};

// ---------- 服务商 ----------

/** 服务商相关服务 */
export const providerService = {
  async list(): Promise<ProviderConfig[]> {
    if (!isTauri) {
      await delay();
      return [...mockProvidersState];
    }
    return call<ProviderConfig[]>("list_providers");
  },

  /** 添加自定义服务商 */
  async add(provider: ProviderConfig): Promise<ProviderConfig> {
    if (!isTauri) {
      await delay();
      mockProvidersState = [...mockProvidersState, provider];
      return provider;
    }
    return call<ProviderConfig>("add_custom_provider", { provider });
  },

  /** 更新自定义服务商 */
  async update(provider: ProviderConfig): Promise<ProviderConfig> {
    if (!isTauri) {
      await delay();
      mockProvidersState = mockProvidersState.map((p) => (p.id === provider.id ? provider : p));
      return provider;
    }
    return call<ProviderConfig>("update_provider", { provider });
  },

  /** 删除自定义服务商 */
  async delete(id: string): Promise<void> {
    if (!isTauri) {
      await delay();
      mockProvidersState = mockProvidersState.filter((p) => p.id !== id);
      return;
    }
    await call("delete_provider", { id });
  },
};

// ---------- 设置与统计 ----------

/** 应用设置服务 */
export const settingsService = {
  async get(): Promise<AppSettings> {
    if (!isTauri) {
      await delay();
      return { theme: "system", autoLockMinutes: 5, language: "zh-CN" };
    }
    return call<AppSettings>("get_settings");
  },

  async update(settings: AppSettings): Promise<AppSettings> {
    if (!isTauri) {
      await delay();
      return settings;
    }
    return call<AppSettings>("update_settings", { settings });
  },
};

/** 仪表盘统计 */
export const dashboardService = {
  async getStats(): Promise<DashboardStats> {
    if (!isTauri) {
      await delay();
      return {
        totalKeys: mockKeys.length,
        totalProviders: mockProvidersState.length,
        recentUsage: mockKeys.filter((k) => k.lastUsedAt).length,
      };
    }
    return call<DashboardStats>("get_dashboard_stats");
  },
};

/** 应用级信息 */
export const appService = {
  async getVersion(): Promise<string> {
    if (!isTauri) return "0.1.0 (browser preview)";
    return call<string>("get_app_version");
  },
};
