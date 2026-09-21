import { useEffect, useState } from "react";
import { Gauge, KeyRound, Loader2, RefreshCw, TrendingDown, TrendingUp } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { apiKeyService, usageService } from "@/services/api";
import type { ApiKeySummary, UsageReport, UsageSnapshot } from "@/services/types";

const errorMessage = (e: unknown) => (e instanceof Error ? e.message : String(e));

/** 金额展示：CNY/USD 加符号，配额类原样 */
function fmtMoney(v: number, currency: string): string {
  if (currency === "CNY") return `¥${v.toFixed(2)}`;
  if (currency === "USD") return `$${v.toFixed(2)}`;
  return `${currency ? `${currency} ` : ""}${v.toFixed(2)}`;
}

function fmtTokens(v: number): string {
  return v.toLocaleString("zh-CN");
}

/** 余额趋势迷你折线（内联 SVG，无图表库依赖） */
function Sparkline({ values }: { values: number[] }) {
  if (values.length < 2) return null;
  const w = 160;
  const h = 36;
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const pts = values
    .map(
      (v, i) =>
        `${((i / (values.length - 1)) * w).toFixed(1)},${(h - 3 - ((v - min) / span) * (h - 6)).toFixed(1)}`,
    )
    .join(" ");
  return (
    <svg width={w} height={h} className="shrink-0 text-primary/70" aria-hidden>
      <polyline points={pts} fill="none" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}

export default function Usage() {
  const [keys, setKeys] = useState<ApiKeySummary[]>([]);
  const [reports, setReports] = useState<Record<string, UsageReport>>({});
  const [history, setHistory] = useState<Record<string, UsageSnapshot[]>>({});
  const [loadingId, setLoadingId] = useState<string | null>(null);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [refreshingAll, setRefreshingAll] = useState(false);

  useEffect(() => {
    apiKeyService.list().then(setKeys).catch((e) => alert(errorMessage(e)));
  }, []);

  const fetchOne = async (keyId: string) => {
    setLoadingId(keyId);
    setErrors((prev) => {
      const next = { ...prev };
      delete next[keyId];
      return next;
    });
    try {
      const report = await usageService.fetch(keyId);
      setReports((prev) => ({ ...prev, [keyId]: report }));
      const snapshots = await usageService.history(keyId);
      setHistory((prev) => ({ ...prev, [keyId]: snapshots }));
    } catch (e) {
      setErrors((prev) => ({ ...prev, [keyId]: errorMessage(e) }));
    } finally {
      setLoadingId(null);
    }
  };

  const fetchAll = async () => {
    setRefreshingAll(true);
    // 顺序查询，避免同时打满各家限流
    for (const key of keys) {
      await fetchOne(key.id);
    }
    setRefreshingAll(false);
  };

  const anyResult = Object.keys(reports).length > 0 || Object.keys(errors).length > 0;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">额度用量</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            手动查询各厂商余额与用量，结果快照保存在保险库中
          </p>
        </div>
        <Button
          size="sm"
          variant="outline"
          disabled={keys.length === 0 || loadingId !== null || refreshingAll}
          onClick={() => void fetchAll()}
        >
          {refreshingAll ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="mr-2 h-4 w-4" />
          )}
          全部查询
        </Button>
      </div>

      {keys.length === 0 && (
        <Card>
          <CardContent className="flex flex-col items-center gap-3 py-12 text-center">
            <div className="flex h-12 w-12 items-center justify-center rounded-full bg-secondary">
              <Gauge className="h-6 w-6 text-muted-foreground" />
            </div>
            <div>
              <p className="text-sm font-medium">还没有可查询的密钥</p>
              <p className="mt-1 text-xs text-muted-foreground">
                请先在「密钥管理」中添加 API Key，再回到这里查询额度
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      <div className="space-y-3">
        {keys.map((key) => {
          const report = reports[key.id];
          const snapshots = history[key.id] ?? [];
          const error = errors[key.id];
          const loading = loadingId === key.id;
          // 余额快照序列（历史为时间倒序，画图时转正序）
          const balances = [...snapshots]
            .reverse()
            .filter((s) => s.totalBalance != null)
            .map((s) => s.totalBalance as number);
          const delta =
            snapshots.length >= 2 &&
            snapshots[0].totalBalance != null &&
            snapshots[1].totalBalance != null
              ? (snapshots[0].totalBalance as number) - (snapshots[1].totalBalance as number)
              : null;
          return (
            <Card key={key.id}>
              <CardContent className="py-4">
                <div className="flex flex-col items-start gap-3 md:flex-row md:items-center md:justify-between">
                  <div className="flex min-w-0 items-center gap-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-secondary">
                      <KeyRound className="h-5 w-5 text-muted-foreground" />
                    </div>
                    <div>
                      <p className="text-sm font-medium">{key.name}</p>
                      <div className="mt-1 flex items-center gap-2">
                        <span className="text-xs text-muted-foreground">{key.providerName}</span>
                        {report?.balance && (
                          <Badge variant="secondary" className="text-xs">
                            {report.balance.total != null
                              ? fmtMoney(report.balance.total, report.balance.currency)
                              : report.balance.currency}
                          </Badge>
                        )}
                        {report?.unsupportedReason && (
                          <Badge variant="outline" className="text-xs">不支持查询</Badge>
                        )}
                        {error && (
                          <Badge variant="destructive" className="text-xs">查询失败</Badge>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex items-center gap-4">
                    {delta != null && Math.abs(delta) >= 0.005 && (
                      <span
                        className={`flex items-center gap-1 text-xs ${
                          delta < 0 ? "text-destructive" : "text-muted-foreground"
                        }`}
                        title="与上一次查询的余额差值"
                      >
                        {delta < 0 ? (
                          <TrendingDown className="h-3.5 w-3.5" />
                        ) : (
                          <TrendingUp className="h-3.5 w-3.5" />
                        )}
                        {delta > 0 ? "+" : ""}
                        {delta.toFixed(2)}
                      </span>
                    )}
                    <Sparkline values={balances} />
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={loading || refreshingAll || loadingId !== null}
                      onClick={() => void fetchOne(key.id)}
                    >
                      {loading ? (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      ) : (
                        <RefreshCw className="mr-2 h-4 w-4" />
                      )}
                      查询
                    </Button>
                  </div>
                </div>

                {(report || error) && (
                  <>
                    <Separator className="my-3" />
                    <div className="space-y-3 text-sm">
                      {error && <p className="text-xs text-destructive">{error}</p>}
                      {report?.unsupportedReason && (
                        <p className="text-xs text-muted-foreground">{report.unsupportedReason}</p>
                      )}
                      {report?.balance && (
                        <div className="flex flex-wrap gap-x-8 gap-y-1 text-xs text-muted-foreground">
                          {report.balance.granted != null && (
                            <span>
                              赠送/券：
                              {fmtMoney(report.balance.granted, report.balance.currency)}
                            </span>
                          )}
                          {report.balance.toppedUp != null && (
                            <span>
                              充值总额：
                              {fmtMoney(report.balance.toppedUp, report.balance.currency)}
                            </span>
                          )}
                          {report.balance.extra.map((line) => (
                            <span key={line.label}>
                              {line.label}：{line.value}
                            </span>
                          ))}
                          <span className="ml-auto">
                            查询于 {new Date(report.fetchedAt).toLocaleString("zh-CN")}
                          </span>
                        </div>
                      )}
                      {report?.tokenUsage && report.tokenUsage.length > 0 && (
                        <table className="w-full max-w-md text-xs">
                          <thead>
                            <tr className="border-b text-left text-muted-foreground">
                              <th className="py-1 font-medium">模型</th>
                              <th className="py-1 text-right font-medium">Token 用量</th>
                            </tr>
                          </thead>
                          <tbody>
                            {report.tokenUsage.map((u) => (
                              <tr key={u.model} className="border-b last:border-0">
                                <td className="py-1 font-mono">{u.model}</td>
                                <td className="py-1 text-right">
                                  {u.totalTokens != null ? fmtTokens(u.totalTokens) : "—"}
                                </td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                      )}
                    </div>
                  </>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>

      {keys.length > 0 && !anyResult && (
        <p className="text-center text-xs text-muted-foreground">
          点击「查询」获取该密钥的额度与用量；厂商不支持公开查询时会给出说明
        </p>
      )}
    </div>
  );
}
