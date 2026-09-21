import { useEffect, useState } from "react";
import { NavLink, Outlet } from "react-router-dom";
import { LayoutDashboard, KeyRound, Server, Settings, Lock, Gauge } from "lucide-react";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { appService } from "@/services/api";
import { useVault } from "@/hooks/useVault";
import dragonLogo from "@/assets/dragon-logo.svg";
import { WindowControls } from "@/components/layout/WindowControls";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "仪表盘" },
  { to: "/keys", icon: KeyRound, label: "密钥管理" },
  { to: "/providers", icon: Server, label: "服务商" },
  { to: "/usage", icon: Gauge, label: "用量" },
  { to: "/settings", icon: Settings, label: "设置" },
];

export function AppLayout() {
  const { lock } = useVault();
  const [version, setVersion] = useState("v0.1.0");

  useEffect(() => {
    appService.getVersion().then((v) => setVersion(`v${v}`)).catch(() => {});
  }, []);

  return (
    <div className="flex h-dvh overflow-hidden bg-background">
      {/* Sidebar（仅桌面端） */}
      <aside className="hidden w-60 flex-col border-r bg-sidebar select-none md:flex">
        <div data-tauri-drag-region className="flex items-center gap-2.5 px-6 py-4 cursor-default">
          <img src={dragonLogo} alt="Dragon Vault" className="h-7 w-7 dark:invert pointer-events-none" />
          <span className="text-base font-bold tracking-tight pointer-events-none">Dragon Vault</span>
        </div>
        <nav className="flex-1 space-y-1 px-3 py-2">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-muted-foreground hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground"
                )
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>
        <Separator />
        <div className="flex items-center justify-between px-6 py-3.5">
          <span className="text-xs text-muted-foreground">{version}</span>
          <Button variant="ghost" size="icon" className="h-8 w-8" title="锁定保险库" onClick={() => void lock()}>
            <Lock className="h-4 w-4 text-muted-foreground" />
          </Button>
        </div>
      </aside>

      <div className="flex flex-1 flex-col overflow-hidden">
        {/* 移动端顶栏：品牌 + 锁库，预留刘海安全区 */}
        <header className="border-b bg-sidebar select-none pt-[env(safe-area-inset-top)] md:hidden">
          <div className="flex h-12 items-center justify-between px-4">
            <div className="flex items-center gap-2">
              <img src={dragonLogo} alt="Dragon Vault" className="h-6 w-6 dark:invert" />
              <span className="text-base font-bold tracking-tight">Dragon Vault</span>
            </div>
            <Button variant="ghost" size="icon" className="h-9 w-9" title="锁定保险库" onClick={() => void lock()}>
              <Lock className="h-4 w-4 text-muted-foreground" />
            </Button>
          </div>
        </header>

        {/* 桌面端顶部拖拽条 & 窗控按钮 */}
        <div className="hidden h-9 items-center justify-between select-none md:flex">
          <div data-tauri-drag-region className="flex-1 h-full cursor-default" />
          <WindowControls />
        </div>

        <main className="flex-1 overflow-y-auto bg-background px-4 pb-6 pt-2 md:px-6">
          <Outlet />
        </main>

        {/* 移动端底部 Tab 导航，避让手势条 */}
        <nav className="grid grid-cols-5 border-t bg-sidebar select-none pb-[env(safe-area-inset-bottom)] md:hidden">
          {navItems.map(({ to, icon: Icon, label }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                cn(
                  "flex flex-col items-center justify-center gap-1 py-2.5 text-xs font-medium transition-colors",
                  isActive
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-muted-foreground"
                )
              }
            >
              <Icon className="h-5 w-5" />
              {label}
            </NavLink>
          ))}
        </nav>
      </div>
    </div>
  );
}
