import { useState } from "react";
import { Moon, Sun, Monitor, Upload, Download, Info } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";

type ThemeMode = "light" | "dark" | "system";

export default function Settings() {
  const [theme, setTheme] = useState<ThemeMode>("system");
  const [autoLock, setAutoLock] = useState("5");

  const themeOptions: { value: ThemeMode; icon: typeof Sun; label: string }[] = [
    { value: "light", icon: Sun, label: "浅色" },
    { value: "dark", icon: Moon, label: "深色" },
    { value: "system", icon: Monitor, label: "跟随系统" },
  ];

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
                variant={theme === value ? "default" : "outline"}
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
              value={autoLock}
              onChange={(e) => setAutoLock(e.target.value)}
            />
          </div>
        </CardContent>
      </Card>

      {/* Data */}
      <Card>
        <CardHeader>
          <CardTitle>数据管理</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex gap-3">
            <Button variant="outline" size="sm">
              <Download className="mr-2 h-4 w-4" />
              导出数据
            </Button>
            <Button variant="outline" size="sm">
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
              <p className="text-xs text-muted-foreground">版本 0.1.0 · AGPL-3.0</p>
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
