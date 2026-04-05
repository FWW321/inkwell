import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import {
  Cpu,
  RefreshCw,
  ChevronDown,
  Zap,
  Trash2,
  Plus,
  Pencil,
  Star,
  ArrowLeft,
} from "lucide-react";
import { aiApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { AiConfig } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Popover, PopoverTrigger, PopoverContent } from "@/components/ui/popover";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";

const ModelCard = ({
  model,
  onEdit,
  onDelete,
  onSetDefault,
}: {
  model: AiConfig;
  onEdit: (m: AiConfig) => void;
  onDelete: (id: string) => void;
  onSetDefault: (id: string) => void;
}) => {
  return (
    <Card className={cn("transition-all duration-150", model.is_default && "ring-primary/30 border-primary/20")}>
      <div className="flex items-center gap-3 px-4 py-3.5">
        <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/12">
          <Cpu className="size-4 text-primary/80" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <p className="text-sm font-medium text-card-foreground truncate">{model.name || model.model}</p>
            {model.is_default && (
              <span className="shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium bg-primary/15 text-primary">默认</span>
            )}
          </div>
          <p className="mt-0.5 text-xs text-muted-foreground/60 truncate">{model.model}</p>
          <p className="mt-0.5 text-[11px] text-muted-foreground/40 truncate">{model.base_url}</p>
        </div>
        <div className="flex items-center gap-1">
          {!model.is_default && (
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={() => onSetDefault(model.id)}
              className="text-muted-foreground/50 hover:text-amber-500"
            >
              <Star className="size-3.5" />
            </Button>
          )}
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={() => onEdit(model)}
            className="text-muted-foreground/50 hover:text-foreground"
          >
            <Pencil className="size-3.5" />
          </Button>
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={() => onDelete(model.id)}
            className="text-muted-foreground/50 hover:text-destructive"
          >
            <Trash2 className="size-3.5" />
          </Button>
        </div>
      </div>
    </Card>
  );
};

const ModelCombobox = ({
  value,
  onChange,
  models,
  loading,
}: {
  value: string;
  onChange: (val: string) => void;
  models: string[];
  loading: boolean;
}) => {
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [inputMode, setInputMode] = useState(false);

  const filtered = models.filter((m) =>
    m.toLowerCase().includes(search.toLowerCase()),
  );

  const showCustomOption =
    search.trim() &&
    !models.some((m) => m.toLowerCase() === search.toLowerCase()) &&
    !filtered.some((m) => m === search.trim());

  const selectModel = (m: string) => {
    onChange(m);
    setOpen(false);
    setSearch("");
  };

  return (
    <div className="flex gap-2">
      <div className="flex-1">
        {inputMode ? (
          <Input
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            placeholder="输入自定义模型名称..."
          />
        ) : (
          <Popover open={open} onOpenChange={setOpen}>
            <PopoverTrigger
              render={
                <Button
                  type="button"
                  variant="outline"
                  className="flex w-full justify-between font-normal"
                />
              }
            >
              <span className={cn(!value && "text-muted-foreground/60")}>
                {value || "选择模型..."}
              </span>
              <ChevronDown className={cn("size-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </PopoverTrigger>
            <PopoverContent
              align="start"
              className="w-[--anchor-width] p-0"
            >
              <div className="border-b border-border p-2.5">
                <Input
                  type="text"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="搜索模型..."
                  autoFocus
                />
              </div>
              <div className="max-h-48 overflow-y-auto p-1">
                {filtered.length === 0 && !showCustomOption && (
                  <div className="px-3 py-3 text-sm text-muted-foreground">
                    {models.length === 0 ? "点击「获取」拉取模型列表" : "无匹配模型"}
                  </div>
                )}
                {showCustomOption && (
                  <button
                    type="button"
                    onClick={() => selectModel(search.trim())}
                    className="flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm text-primary transition-colors hover:bg-secondary"
                  >
                    使用「{search.trim()}」
                  </button>
                )}
                {filtered.map((m) => (
                  <button
                    key={m}
                    type="button"
                    onClick={() => selectModel(m)}
                    className={cn(
                      "flex w-full items-center rounded-md px-3 py-2 text-left text-sm transition-colors hover:bg-secondary",
                      m === value && "font-medium text-primary",
                    )}
                  >
                    {m}
                  </button>
                ))}
              </div>
            </PopoverContent>
          </Popover>
        )}
      </div>
      <Button
        type="button"
        variant="outline"
        size="sm"
        onClick={() => setInputMode(!inputMode)}
        title={inputMode ? "从列表选择" : "手动输入"}
      >
        {inputMode ? "列表" : "手动"}
      </Button>
      {loading && (
        <div className="flex items-center">
          <RefreshCw className="size-3.5 animate-spin text-primary/60" />
        </div>
      )}
    </div>
  );
};

const ModelForm = ({
  initial,
  onSave,
  onCancel,
}: {
  initial?: AiConfig | null;
  onSave: (data: { name: string; apiKey: string; model: string; baseUrl: string }) => void;
  onCancel: () => void;
}) => {
  const [name, setName] = useState(initial?.name ?? "");
  const [apiKey, setApiKey] = useState(initial?.api_key ?? "");
  const [baseUrl, setBaseUrl] = useState(initial?.base_url ?? "https://api.openai.com/v1");
  const [model, setModel] = useState(initial?.model ?? "");
  const [models, setModels] = useState<string[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const isEdit = !!initial;

  const handleFetchModels = async () => {
    if (!apiKey.trim()) return;
    setModelsLoading(true);
    try {
      const list = await aiApi.fetchAvailableModels(apiKey.trim(), baseUrl.trim());
      setModels(list);
    } catch (err) {
      console.error("Failed to fetch models:", err);
    } finally {
      setModelsLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !model.trim()) return;
    setSaving(true);
    try {
      await onSave({ name: name.trim(), apiKey: apiKey.trim(), model: model.trim(), baseUrl: baseUrl.trim() });
    } finally {
      setSaving(false);
    }
  };

  useEffect(() => {
    if (apiKey.trim() && baseUrl.trim()) {
      handleFetchModels();
    }
  }, []);

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <div className="flex flex-col gap-1.5">
        <Label>配置名称</Label>
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="如：GPT-4o、DeepSeek V3"
          autoFocus
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>API Key</Label>
        <Input
          type="password"
          value={apiKey}
          onChange={(e) => setApiKey(e.target.value)}
          placeholder="sk-..."
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>API Base URL</Label>
        <Input
          type="text"
          value={baseUrl}
          onChange={(e) => setBaseUrl(e.target.value)}
          placeholder="https://api.openai.com/v1"
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>模型</Label>
        <ModelCombobox
          value={model}
          onChange={setModel}
          models={models}
          loading={modelsLoading}
        />
        <button
          type="button"
          onClick={handleFetchModels}
          className="self-start text-xs text-primary/60 hover:text-primary transition-colors"
        >
          重新获取模型列表
        </button>
      </div>
      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>取消</Button>
        <Button type="submit" disabled={saving || !name.trim() || !model.trim()}>
          {saving ? "保存中..." : isEdit ? "更新" : "添加"}
        </Button>
      </DialogFooter>
    </form>
  );
};

const SettingsPage = () => {
  const [models, setModels] = useState<AiConfig[]>([]);
  const navigate = useNavigate();
  const [loading, setLoading] = useState(true);
  const [showForm, setShowForm] = useState(false);
  const [editingModel, setEditingModel] = useState<AiConfig | null>(null);

  const loadModels = async () => {
    try {
      const data = await aiApi.listModels();
      setModels(data);
    } catch (err) {
      console.error("Failed to load models:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadModels();
  }, []);

  const handleCreate = async (data: { name: string; apiKey: string; model: string; baseUrl: string }) => {
    await aiApi.createModel(data.name, data.apiKey, data.model, data.baseUrl);
    await loadModels();
    setShowForm(false);
  };

  const handleUpdate = async (data: { name: string; apiKey: string; model: string; baseUrl: string }) => {
    if (!editingModel) return;
    await aiApi.updateModel(editingModel.id, data.name, data.apiKey, data.model, data.baseUrl);
    await loadModels();
    setEditingModel(null);
  };

  const handleDelete = async (id: string) => {
    await aiApi.deleteModel(id);
    setModels((prev) => prev.filter((m) => m.id !== id));
  };

  const handleSetDefault = async (id: string) => {
    await aiApi.setDefault(id);
    setModels((prev) => prev.map((m) => ({ ...m, is_default: m.id === id })));
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 px-5 h-11 shrink-0">
        <Button variant="ghost" size="sm" className="size-8 p-0" onClick={() => navigate("/")}>
          <ArrowLeft className="size-4" />
        </Button>
        <h1 className="text-sm font-medium text-foreground">设置</h1>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-6">
        <div className="mx-auto max-w-2xl">
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2.5">
                  <div className="flex size-8 items-center justify-center rounded-lg bg-primary/12">
                    <Zap className="size-4 text-primary/80" />
                  </div>
                  <div>
                    <CardTitle>AI 模型配置</CardTitle>
                    <CardDescription>管理多个 AI 模型配置</CardDescription>
                  </div>
                </div>
                <Button onClick={() => { setEditingModel(null); setShowForm(true); }} data-icon="inline-start">
                  <Plus />
                  添加模型
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              {loading ? (
                <div className="flex h-20 items-center justify-center">
                  <Spinner className="size-5 text-primary" />
                </div>
              ) : models.length === 0 ? (
                <div className="flex flex-col items-center gap-2 py-8 text-muted-foreground/60">
                  <Cpu className="size-8 text-muted-foreground/30" />
                  <p className="text-sm">还没有配置模型</p>
                  <Button variant="outline" size="sm" onClick={() => { setEditingModel(null); setShowForm(true); }} data-icon="inline-start">
                    <Plus />
                    添加模型
                  </Button>
                </div>
              ) : (
                <div className="flex flex-col gap-2">
                  {models.map((m) => (
                    <ModelCard
                      key={m.id}
                      model={m}
                      onEdit={(model) => { setEditingModel(model); setShowForm(true); }}
                      onDelete={handleDelete}
                      onSetDefault={handleSetDefault}
                    />
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      <Dialog open={showForm} onOpenChange={(isOpen) => { if (!isOpen) { setShowForm(false); setEditingModel(null); } }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{editingModel ? "编辑模型" : "添加模型"}</DialogTitle>
            <DialogDescription>{editingModel ? "修改 AI 模型配置" : "添加一个新的 AI 模型配置"}</DialogDescription>
          </DialogHeader>
          <ModelForm
            key={editingModel?.id ?? "new"}
            initial={editingModel}
            onSave={editingModel ? handleUpdate : handleCreate}
            onCancel={() => { setShowForm(false); setEditingModel(null); }}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default SettingsPage;
