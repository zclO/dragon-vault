import { NavLink, Outlet } from "react-router-dom";
import { LayoutDashboard, KeyRound, Server, Settings } from "lucide-react";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import dragonLogo from "@/assets/dragon-logo.svg";

const navItems = [
  { to: "/", icon: LayoutDashboard, label: "仪表盘" },
  { to: "/keys", icon: KeyRound, label: "密钥管理" },
  { to: "/providers", icon: Server, label: "服务商" },
  { to: "/settings", icon: Settings, label: "设置" },
];

export function AppLayout() {
  return (
    <div className="flex h-screen overflow-hidden">
      {/* Sidebar */}
      <aside className="flex w-60 flex-col border-r bg-sidebar">
        <div className="flex items-center gap-2 px-6 py-5">
          <img src={dragonLogo} alt="Dragon Vault" className="h-8 w-8" />
          <span className="text-lg font-bold tracking-tight">Dragon Vault</span>
        </div>
        <Separator />
        <nav className="flex-1 space-y-1 px-3 py-4">
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
        <div className="px-6 py-4 text-xs text-muted-foreground">v0.1.0</div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-y-auto bg-background p-6">
        <Outlet />
      </main>
    </div>
  );
}
