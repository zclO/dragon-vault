import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { KeyRound, Server, Zap, Plus, Download } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { dashboardService, apiKeyService } from "@/services/api";
import type { DashboardStats, ApiKeySummary } from "@/services/types";

export default function Dashboard() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [recentKeys, setRecentKeys] = useState<ApiKeySummary[]>([]);

  useEffect(() => {
    dashboardService.getStats().then(setStats);
    apiKeyService.list().then((keys) =>
      setRecentKeys(
        keys
          .filter((k) => k.lastUsedAt)
          .sort((a, b) => new Date(b.lastUsedAt!).getTime() - new Date(a.lastUsedAt!).getTime())
          .slice(0, 5)
      )
    );
  }, []);

  const statCards = stats
    ? [
        { label: "已存密钥", value: stats.totalKeys, icon: KeyRound },
        { label: "服务商", value: stats.totalProviders, icon: Server },
        { label: "近 7 天使用", value: stats.recentUsage, icon: Zap },
      ]
    : [];

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="hidden text-2xl font-bold md:block">仪表盘</h1>
        <div className="flex gap-2">
          <Link to="/keys">
            <Button size="sm">
              <Plus className="mr-2 h-4 w-4" />
              添加密钥
            </Button>
          </Link>
          <Button variant="outline" size="sm">
            <Download className="mr-2 h-4 w-4" />
            导入备份
          </Button>
        </div>
      </div>

      {/* Stats */}
      <div className="grid gap-4 sm:grid-cols-3">
        {statCards.map(({ label, value, icon: Icon }) => (
          <Card key={label}>
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">{label}</CardTitle>
              <Icon className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <p className="text-3xl font-bold">{value}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Recent Keys */}
      <Card>
        <CardHeader>
          <CardTitle>最近使用</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {recentKeys.map((key) => (
              <div key={key.id} className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <KeyRound className="h-4 w-4 text-muted-foreground" />
                  <div>
                    <p className="text-sm font-medium">{key.name}</p>
                    <p className="text-xs text-muted-foreground">{key.providerName}</p>
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  {key.tags.map((tag) => (
                    <Badge key={tag} variant="secondary" className="text-xs">
                      {tag}
                    </Badge>
                  ))}
                  <span className="text-xs text-muted-foreground">
                    {new Date(key.lastUsedAt!).toLocaleDateString()}
                  </span>
                </div>
              </div>
            ))}
            {recentKeys.length === 0 && (
              <p className="py-4 text-center text-sm text-muted-foreground">暂无使用记录</p>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
