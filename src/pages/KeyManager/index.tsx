import { useEffect, useState } from "react";
import { KeyRound, Copy, Check, Pencil, Trash2, Plus, Search, Zap, Loader2 } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { apiKeyService, providerService } from "@/services/api";
import type { ApiKeySummary, ProviderConfig } from "@/services/types";

type TestState = "testing" | "ok" | "fail";

const errorMessage = (e: unknown) => (e instanceof Error ? e.message : String(e));

export default function KeyManager() {
  const [keys, setKeys] = useState<ApiKeySummary[]>([]);
  const [providers, setProviders] = useState<ProviderConfig[]>([]);
  const [search, setSearch] = useState("");
  const [filterProvider, setFilterProvider] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [newKey, setNewKey] = useState({ name: "", providerId: "", value: "", tags: "" });
  const [createError, setCreateError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [editing, setEditing] = useState<ApiKeySummary | null>(null);
  const [editForm, setEditForm] = useState({ name: "", value: "", tags: "" });
  const [editError, setEditError] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);
  const [testStates, setTestStates] = useState<Record<string, TestState>>({});

  const reload = () => {
    apiKeyService.list().then(setKeys).catch((e) => alert(errorMessage(e)));
    providerService.list().then(setProviders).catch(() => {});
  };

  useEffect(reload, []);

  const filtered = keys.filter((k) => {
    const matchSearch =
      k.name.toLowerCase().includes(search.toLowerCase()) ||
      k.tags.some((t) => t.toLowerCase().includes(search.toLowerCase()));
    const matchProvider = !filterProvider || k.providerId === filterProvider;
    return matchSearch && matchProvider;
  });

  const parseTags = (raw: string) =>
    raw
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

  const handleCreate = async () => {
    if (!newKey.name || !newKey.providerId || !newKey.value) return;
    setCreateError(null);
    setSaving(true);
    try {
      await apiKeyService.create({
        name: newKey.name,
        providerId: newKey.providerId,
        value: newKey.value,
        tags: parseTags(newKey.tags),
      });
      setCreateOpen(false);
      setNewKey({ name: "", providerId: "", value: "", tags: "" });
      reload();
    } catch (e) {
      setCreateError(errorMessage(e));
    } finally {
      setSaving(false);
    }
  };

  const openEdit = (key: ApiKeySummary) => {
    setEditing(key);
    setEditForm({ name: key.name, value: "", tags: key.tags.join(", ") });
    setEditError(null);
  };

  const handleUpdate = async () => {
    if (!editing) return;
    setEditError(null);
    setSaving(true);
    try {
      await apiKeyService.update({
        id: editing.id,
        name: editForm.name,
        tags: parseTags(editForm.tags),
        // 留空表示不修改密钥值
        value: editForm.value.trim() || undefined,
      });
      setEditing(null);
      reload();
    } catch (e) {
      setEditError(errorMessage(e));
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm("确定删除该密钥？此操作不可撤销。")) return;
    try {
      await apiKeyService.delete(id);
      setKeys((prev) => prev.filter((k) => k.id !== id));
    } catch (e) {
      alert(errorMessage(e));
    }
  };

  const handleCopy = async (id: string) => {
    try {
      const value = await apiKeyService.reveal(id);
      await navigator.clipboard.writeText(value);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 2000);
      reload();
    } catch (e) {
      alert(errorMessage(e));
    }
  };

  const handleTest = async (id: string) => {
    setTestStates((prev) => ({ ...prev, [id]: "testing" }));
    try {
      const ok = await apiKeyService.testConnection(id);
      setTestStates((prev) => ({ ...prev, [id]: ok ? "ok" : "fail" }));
    } catch {
      setTestStates((prev) => ({ ...prev, [id]: "fail" }));
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">密钥管理</h1>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          添加密钥
        </Button>
      </div>

      {/* Search & Filter */}
      <div className="flex gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜索密钥名称或标签..."
            className="pl-9"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>
        <select
          className="flex h-9 items-center rounded-md border border-input bg-transparent px-3 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          value={filterProvider}
          onChange={(e) => setFilterProvider(e.target.value)}
        >
          <option value="">全部服务商</option>
          {providers.map((p) => (
            <option key={p.id} value={p.id}>
              {p.name}
            </option>
          ))}
        </select>
      </div>

      {/* Key List */}
      <div className="space-y-3">
        {filtered.map((key) => {
          const testState = testStates[key.id];
          return (
            <Card key={key.id}>
              <CardContent className="flex items-center justify-between py-4">
                <div className="flex items-center gap-4">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-secondary">
                    <KeyRound className="h-5 w-5 text-muted-foreground" />
                  </div>
                  <div>
                    <p className="text-sm font-medium">{key.name}</p>
                    <div className="mt-1 flex items-center gap-2">
                      <span className="text-xs text-muted-foreground">{key.providerName}</span>
                      <span className="text-xs text-muted-foreground">·</span>
                      <span className="font-mono text-xs text-muted-foreground">
                        ••••{key.id.slice(-4)}
                      </span>
                      {testState === "ok" && (
                        <Badge variant="secondary" className="text-xs">连通正常</Badge>
                      )}
                      {testState === "fail" && (
                        <Badge variant="destructive" className="text-xs">连通失败</Badge>
                      )}
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <div className="flex gap-1">
                    {key.tags.map((tag) => (
                      <Badge key={tag} variant="secondary" className="text-xs">
                        {tag}
                      </Badge>
                    ))}
                  </div>
                  <div className="flex gap-1">
                    <Button
                      variant="ghost"
                      size="icon"
                      title="测试连通性"
                      disabled={testState === "testing"}
                      onClick={() => void handleTest(key.id)}
                    >
                      {testState === "testing" ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        <Zap className="h-4 w-4" />
                      )}
                    </Button>
                    <Button variant="ghost" size="icon" title="复制密钥" onClick={() => void handleCopy(key.id)}>
                      {copiedId === key.id ? (
                        <Check className="h-4 w-4 text-green-600" />
                      ) : (
                        <Copy className="h-4 w-4" />
                      )}
                    </Button>
                    <Button variant="ghost" size="icon" title="编辑" onClick={() => openEdit(key)}>
                      <Pencil className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      title="删除"
                      onClick={() => void handleDelete(key.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                  <span className="min-w-[80px] text-right text-xs text-muted-foreground">
                    {key.lastUsedAt ? new Date(key.lastUsedAt).toLocaleDateString() : "未使用"}
                  </span>
                </div>
              </CardContent>
            </Card>
          );
        })}
        {filtered.length === 0 && (
          <p className="py-12 text-center text-sm text-muted-foreground">没有找到匹配的密钥</p>
        )}
      </div>

      {/* Create Dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>添加新密钥</DialogTitle>
            <DialogDescription>输入 API Key 信息，密钥值将被加密存储。</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">名称</label>
              <Input
                placeholder="例如：OpenAI 主力 Key"
                value={newKey.name}
                onChange={(e) => setNewKey({ ...newKey, name: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">服务商</label>
              <select
                className="flex h-9 w-full rounded-md border border-input bg-transparent px-3 text-sm shadow-sm focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                value={newKey.providerId}
                onChange={(e) => setNewKey({ ...newKey, providerId: e.target.value })}
              >
                <option value="">选择服务商</option>
                {providers.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">密钥值</label>
              <Input
                type="password"
                placeholder="sk-..."
                value={newKey.value}
                onChange={(e) => setNewKey({ ...newKey, value: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">标签（逗号分隔）</label>
              <Input
                placeholder="生产, GPT-4"
                value={newKey.tags}
                onChange={(e) => setNewKey({ ...newKey, tags: e.target.value })}
              />
            </div>
            {createError && <p className="text-sm text-destructive">{createError}</p>}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setCreateOpen(false)}>
              取消
            </Button>
            <Button onClick={() => void handleCreate()} disabled={saving}>
              {saving ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              保存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Edit Dialog */}
      <Dialog open={editing !== null} onOpenChange={(open) => !open && setEditing(null)}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>编辑密钥</DialogTitle>
            <DialogDescription>修改名称与标签；如需轮换密钥值，输入新值留空则保持不变。</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">名称</label>
              <Input
                value={editForm.name}
                onChange={(e) => setEditForm({ ...editForm, name: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">新密钥值（可选）</label>
              <Input
                type="password"
                placeholder="留空表示不修改"
                value={editForm.value}
                onChange={(e) => setEditForm({ ...editForm, value: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">标签（逗号分隔）</label>
              <Input
                value={editForm.tags}
                onChange={(e) => setEditForm({ ...editForm, tags: e.target.value })}
              />
            </div>
            {editError && <p className="text-sm text-destructive">{editError}</p>}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setEditing(null)}>
              取消
            </Button>
            <Button onClick={() => void handleUpdate()} disabled={saving}>
              {saving ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
              保存
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
