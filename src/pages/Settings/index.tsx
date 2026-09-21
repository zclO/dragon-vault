import { useEffect, useState } from "react";
import { Moon, Sun, Monitor, Upload, Download, Info, Loader2, Save } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import { appService, biometricService, settingsService } from "@/services/api";
import type { AppSettings, BiometricStatus, ThemeMode } from "@/services/types";
import { applyTheme } from "@/hooks/useVault";

const errorMessage = (e: unknown) => (e instanceof Error ? e.message : String(e));

const DEFAULT_SETTINGS: AppSettings = { theme: "system", autoLockMinutes: 5, language: "zh-CN" };

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [savedTip, setSavedTip] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [version, setVersion] = useState("0.1.0");
  const [bio, setBio] = useState<BiometricStatus | null>(null);
  const [bioBusy, setBioBusy] = useState(false);

  useEffect(() => {
    settingsService
      .get()
      .then((s) => {
        setSettings(s);
        applyTheme(s.theme);
      })
      .catch((e) => setError(errorMessage(e)))
      .finally(() => setLoading(false));
    appService.getVersion().then(setVersion).catch(() => {});
    biometricService.getStatus().then(setBio).catch(() => setBio(null));
  }, []);

  const handleSave = async () => {
    const minutes = Math.min(60, Math.max(1, Math.round(settings.autoLockMinutes) || 5));
    setError(null);
    setSaving(true);
    try {
      const saved = await settingsService.update({ ...settings, autoLockMinutes: minutes });
      setSettings(saved);
      setSavedTip(true);
      setTimeout(() => setSavedTip(false), 2000);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setSaving(false);
    }
  };

  const setTheme = (theme: ThemeMode) => {
    setSettings({ ...settings, theme });
    applyTheme(theme);
  };

  /** 开启/关闭指纹解锁（开启需已解锁，后端将主密钥封存进安全硬件） */
  const handleToggleBiometric = async () => {
    if (!bio) return;
    setBioBusy(true);
    setError(null);
    try {
      if (bio.enrolled) {
        await biometricService.disable();
      } else {
        await biometricService.enable();
      }
      setBio(await biometricService.getStatus());
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBioBusy(false);
    }
  };

  const themeOptions: { value: ThemeMode; icon: typeof Sun; label: string }[] = [
    { value: "light", icon: Sun, label: "浅色" },
    { value: "dark", icon: Moon, label: "深色" },
    { value: "system", icon: Monitor, label: "跟随系统" },
  ];

  if (loading) {
    return (
      <div className="flex justify-center py-16">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold">设置</h1>

      {/* Theme */}
      <Card>
        <CardHeader>
          <CardTitle>外观</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="mb-3 text-sm text-muted-foreground">主题模式</p>
          <div className="flex gap-2">
            {themeOptions.map(({ value, icon: Icon, label }) => (
              <Button
                key={value}
                variant={settings.theme === value ? "default" : "outline"}
                size="sm"
                onClick={() => setTheme(value)}
              >
                <Icon className="mr-2 h-4 w-4" />
                {label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Auto Lock */}
      <Card>
        <CardHeader>
          <CardTitle>安全</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4">
            <p className="text-sm text-muted-foreground">自动锁定时间（分钟）</p>
            <Input
              type="number"
              className="w-24"
              min={1}
              max={60}
              value={settings.autoLockMinutes}
              onChange={(e) => setSettings({ ...settings, autoLockMinutes: Number(e.target.value) })}
            />
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            无操作超过该时长后自动锁定保险库，需重新输入主密码。
          </p>
          <Separator className="my-4" />
          <div className="flex items-center justify-between gap-4">
            <div>
              <p className="text-sm">指纹解锁</p>
              <p className="mt-1 text-xs text-muted-foreground">
                {bio === null
                  ? "检测可用性…"
                  : !bio.available
                    ? (bio.reason ?? "当前设备不可用")
                    : bio.enrolled
                      ? "主密钥已封存于设备安全芯片，验证指纹即可解锁"
                      : "将主密钥封存到设备安全芯片，开启后可用指纹直接解锁"}
              </p>
            </div>
            <Button
              variant={bio?.enrolled ? "outline" : "default"}
              size="sm"
              disabled={bio === null || !bio.available || bioBusy}
              onClick={() => void handleToggleBiometric()}
            >
              {bioBusy ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              {bio?.enrolled ? "关闭" : "开启"}
            </Button>
          </div>
          {bio?.available && bio.enrolled && (
            <p className="mt-2 text-xs text-muted-foreground">
              变更系统指纹后封存密钥会失效，需用主密码解锁后重新开启。
            </p>
          )}
        </CardContent>
      </Card>

      {/* Preference actions */}
      <div className="flex items-center gap-3">
        <Button onClick={() => void handleSave()} disabled={saving}>
          {saving ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Save className="mr-2 h-4 w-4" />}
          保存设置
        </Button>
        {savedTip && <span className="text-sm text-green-600">已保存</span>}
      </div>
      {error && <p className="text-sm text-destructive">{error}</p>}

      {/* Data */}
      <Card>
        <CardHeader>
          <CardTitle>数据管理</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex gap-3">
            <Button variant="outline" size="sm" disabled title="即将上线">
              <Download className="mr-2 h-4 w-4" />
              导出数据
            </Button>
            <Button variant="outline" size="sm" disabled title="即将上线">
              <Upload className="mr-2 h-4 w-4" />
              导入数据
            </Button>
          </div>
          <p className="text-xs text-muted-foreground">
            导出数据为加密格式，仅可在 Dragon Vault 中导入。
          </p>
        </CardContent>
      </Card>

      {/* About */}
      <Card>
        <CardHeader>
          <CardTitle>关于</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-3">
            <Info className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium">Dragon Vault</p>
              <p className="text-xs text-muted-foreground">版本 {version} · AGPL-3.0</p>
            </div>
          </div>
          <Separator className="my-4" />
          <p className="text-xs text-muted-foreground">
            安全保存和管理大模型 API Key 的桌面应用。
            <br />
            基于 Tauri 2 + React + Rust 构建。
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
