import type { ApiKeySummary, CreateApiKeyRequest, ProviderConfig, DashboardStats } from "./types";
import { mockApiKeys, mockProviders } from "./mock-data";

/** 模拟网络延迟 */
function delay(ms = 200): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** API Key 相关服务（Mock 实现） */
export const apiKeyService = {
  /** 获取所有 API Key 列表 */
  async list(): Promise<ApiKeySummary[]> {
    await delay();
    return [...mockApiKeys];
  },

  /** 创建新的 API Key */
  async create(_req: CreateApiKeyRequest): Promise<ApiKeySummary> {
    await delay();
    const newKey: ApiKeySummary = {
      id: `key-${Date.now()}`,
      name: _req.name,
      providerId: _req.providerId,
      providerName: mockProviders.find((p) => p.id === _req.providerId)?.name ?? _req.providerId,
      tags: _req.tags ?? [],
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      lastUsedAt: null,
    };
    return newKey;
  },

  /** 删除 API Key */
  async delete(_id: string): Promise<void> {
    await delay();
  },
};

/** 服务商相关服务（Mock 实现） */
export const providerService = {
  async list(): Promise<ProviderConfig[]> {
    await delay();
    return [...mockProviders];
  },
};

/** 仪表盘统计（Mock 实现） */
export const dashboardService = {
  async getStats(): Promise<DashboardStats> {
    await delay();
    return {
      totalKeys: mockApiKeys.length,
      totalProviders: mockProviders.length,
      recentUsage: 23,
    };
  },
};
