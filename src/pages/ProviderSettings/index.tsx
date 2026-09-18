import { useEffect, useState } from "react";
import { Server, Plus, Globe } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
import { providerService } from "@/services/api";
import type { ProviderConfig } from "@/services/types";

export default function ProviderSettings() {
  const [providers, setProviders] = useState<ProviderConfig[]>([]);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [newProvider, setNewProvider] = useState({ name: "", baseUrl: "" });

  useEffect(() => {
    providerService.list().then(setProviders);
  }, []);

  const handleAdd = () => {
    if (!newProvider.name || !newProvider.baseUrl) return;
    const custom: ProviderConfig = {
      id: `custom-${Date.now()}`,
      name: newProvider.name,
      baseUrl: newProvider.baseUrl,
      keyFormatPattern: "",
      models: [],
      isBuiltIn: false,
    };
    setProviders((prev) => [...prev, custom]);
    setDialogOpen(false);
    setNewProvider({ name: "", baseUrl: "" });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">服务商设置</h1>
        <Button size="sm" onClick={() => setDialogOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          添加自定义服务商
        </Button>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        {providers.map((provider) => (
          <Card key={provider.id}>
            <CardHeader className="pb-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-secondary">
                    <Server className="h-5 w-5 text-muted-foreground" />
                  </div>
                  <div>
                    <CardTitle className="text-base">{provider.name}</CardTitle>
                    <div className="mt-1 flex items-center gap-1 text-xs text-muted-foreground">
                      <Globe className="h-3 w-3" />
                      {provider.baseUrl}
                    </div>
                  </div>
                </div>
                {provider.isBuiltIn ? (
                  <Badge variant="secondary">内建</Badge>
                ) : (
                  <Badge variant="outline">自定义</Badge>
                )}
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-1">
                {provider.models.map((model) => (
                  <Badge key={model} variant="outline" className="text-xs font-normal">
                    {model}
                  </Badge>
                ))}
              </div>
              <p className="mt-3 text-xs text-muted-foreground">
                {provider.models.length} 个模型
              </p>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Add Custom Provider Dialog */}
      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>添加自定义服务商</DialogTitle>
            <DialogDescription>添加兼容 OpenAI API 格式的自定义服务商。</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">名称</label>
              <Input
                placeholder="例如：DeepSeek"
                value={newProvider.name}
                onChange={(e) => setNewProvider({ ...newProvider, name: e.target.value })}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Base URL</label>
              <Input
                placeholder="https://api.example.com/v1"
                value={newProvider.baseUrl}
                onChange={(e) => setNewProvider({ ...newProvider, baseUrl: e.target.value })}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              取消
            </Button>
            <Button onClick={handleAdd}>添加</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
