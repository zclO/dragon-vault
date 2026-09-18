import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { settingsService, vaultService } from "@/services/api";
import type { ThemeMode, VaultStatus } from "@/services/types";

/** 应用主题到 <html>（dark 类供 Tailwind 自定义 variant 使用） */
export function applyTheme(theme: ThemeMode): void {
  const dark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);
  document.documentElement.classList.toggle("dark", dark);
}

interface VaultContextValue {
  /** 保险库状态；null 表示尚未完成首次查询 */
  status: VaultStatus | null;
  /** 后端连通性检查中 */
  loading: boolean;
  /** 解锁失败等用户可见错误 */
  error: string | null;
  initialize: (password: string) => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
}

const VaultContext = createContext<VaultContextValue | null>(null);

/** 空闲检测轮询间隔（毫秒） */
const IDLE_CHECK_MS = 30_000;

export function VaultProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<VaultStatus | null>(null);
  const [error, setError] = useState<string | null>(null);
  const lastActivityRef = useRef(Date.now());

  const refresh = useCallback(async () => {
    try {
      setStatus(await vaultService.getStatus());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const initialize = useCallback(
    async (password: string) => {
      setError(null);
      try {
        await vaultService.initialize(password);
        await refresh();
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [refresh],
  );

  const unlock = useCallback(
    async (password: string) => {
      setError(null);
      try {
        await vaultService.unlock(password);
        lastActivityRef.current = Date.now();
        await refresh();
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    [refresh],
  );

  const lock = useCallback(async () => {
    setError(null);
    try {
      await vaultService.lock();
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [refresh]);

  // 解锁后：加载设置 -> 应用主题，并按 autoLockMinutes 启动空闲自动锁定
  const unlocked = status?.unlocked ?? false;
  useEffect(() => {
    if (!unlocked) return;
    let timer: number | undefined;
    let autoLockMinutes = 0;

    settingsService
      .get()
      .then((settings) => {
        applyTheme(settings.theme);
        autoLockMinutes = settings.autoLockMinutes;
      })
      .catch(() => {
        /* 设置加载失败不阻塞使用 */
      });

    const recordActivity = () => {
      lastActivityRef.current = Date.now();
    };
    const checkIdle = () => {
      if (autoLockMinutes > 0 && Date.now() - lastActivityRef.current >= autoLockMinutes * 60_000) {
        void lock();
      }
    };

    window.addEventListener("pointerdown", recordActivity);
    window.addEventListener("keydown", recordActivity);
    timer = window.setInterval(checkIdle, IDLE_CHECK_MS);
    return () => {
      window.removeEventListener("pointerdown", recordActivity);
      window.removeEventListener("keydown", recordActivity);
      if (timer !== undefined) window.clearInterval(timer);
    };
  }, [unlocked, lock]);

  return (
    <VaultContext.Provider value={{ status, loading: status === null, error, initialize, unlock, lock }}>
      {children}
    </VaultContext.Provider>
  );
}

/** 访问保险库全局状态；必须在 VaultProvider 内使用 */
export function useVault(): VaultContextValue {
  const ctx = useContext(VaultContext);
  if (!ctx) throw new Error("useVault 必须在 VaultProvider 内使用");
  return ctx;
}
