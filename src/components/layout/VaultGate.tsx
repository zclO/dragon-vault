import { useState, type FormEvent, type ReactNode } from "react";
import { ShieldCheck, KeyRound, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useVault } from "@/hooks/useVault";
import dragonLogo from "@/assets/dragon-logo.svg";
import { WindowControls } from "@/components/layout/WindowControls";

/** 主密码强度规则（与后端 validate_password_strength 保持一致） */
function validatePasswordStrength(password: string): string | null {
  if (password.length < 8) return "主密码至少需要 8 个字符";
  if (!/[a-zA-Z]/.test(password)) return "主密码需包含字母";
  if (!/[0-9]/.test(password)) return "主密码需包含数字";
  return null;
}

/** 未解锁时拦截主界面，展示初始化 / 解锁屏幕 */
export function VaultGate({ children }: { children: ReactNode }) {
  const { status, loading, error } = useVault();

  if (loading) {
    return (
      <div className="relative flex h-screen items-center justify-center bg-background">
        <div className="absolute top-0 left-0 right-0 flex h-9 items-center justify-between z-50 select-none">
          <div data-tauri-drag-region className="flex-1 h-full cursor-default" />
          <WindowControls />
        </div>
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!status?.unlocked) {
    return <UnlockScreen initialized={status?.initialized ?? false} error={error} />;
  }

  return <>{children}</>;
}

function UnlockScreen({ initialized, error }: { initialized: boolean; error: string | null }) {
  const { initialize, unlock } = useVault();
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [inputError, setInputError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setInputError(null);
    if (!initialized) {
      const strengthError = validatePasswordStrength(password);
      if (strengthError) {
        setInputError(strengthError);
        return;
      }
      if (password !== confirm) {
        setInputError("两次输入的密码不一致");
        return;
      }
      setSubmitting(true);
      await initialize(password);
    } else {
      if (!password) {
        setInputError("请输入主密码");
        return;
      }
      setSubmitting(true);
      await unlock(password);
    }
    // 失败时保留输入供重试；成功后本组件会随解锁状态卸载
    if (!initialized) setSubmitting(false);
  };

  return (
    <div className="relative flex h-screen items-center justify-center bg-background p-6">
      <div className="absolute top-0 left-0 right-0 flex h-9 items-center justify-between z-50 select-none">
        <div data-tauri-drag-region className="flex-1 h-full cursor-default" />
        <WindowControls />
      </div>
      <div className="w-full max-w-sm">
        <div data-tauri-drag-region className="mb-6 flex flex-col items-center gap-3 cursor-default select-none">
          <img src={dragonLogo} alt="Dragon Vault" className="h-16 w-16 dark:invert pointer-events-none" />
          <h1 className="text-xl font-bold tracking-tight pointer-events-none">Dragon Vault</h1>
        </div>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-lg">
              {initialized ? <KeyRound className="h-5 w-5" /> : <ShieldCheck className="h-5 w-5" />}
              {initialized ? "解锁保险库" : "设置主密码"}
            </CardTitle>
            <CardDescription>
              {initialized
                ? "输入主密码解密并加载你的密钥数据。"
                : "首次使用请设置主密码，所有 API Key 将以 AES-256-GCM 加密存储。主密码无法找回，请务必牢记。"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <label className="text-sm font-medium" htmlFor="master-password">
                  主密码
                </label>
                <Input
                  id="master-password"
                  type="password"
                  autoFocus
                  placeholder={initialized ? "输入主密码" : "至少 8 位，包含字母和数字"}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                />
              </div>
              {!initialized && (
                <div className="space-y-2">
                  <label className="text-sm font-medium" htmlFor="confirm-password">
                    确认主密码
                  </label>
                  <Input
                    id="confirm-password"
                    type="password"
                    placeholder="再次输入"
                    value={confirm}
                    onChange={(e) => setConfirm(e.target.value)}
                  />
                </div>
              )}
              {(inputError ?? error) && (
                <p className="text-sm text-destructive">{inputError ?? error}</p>
              )}
              <Button type="submit" className="w-full" disabled={submitting}>
                {submitting ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
                {initialized ? "解锁" : "创建保险库"}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
