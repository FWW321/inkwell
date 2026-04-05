import { useState, useEffect, useRef } from "react";
import { Key, Cpu, Globe, Save, Check, RefreshCw, ChevronDown, Settings, Zap, ChevronRight } from "lucide-react";
import { aiApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { AiConfig } from "@/lib/types";

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
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const filtered = models.filter((m) =>
    m.toLowerCase().includes(search.toLowerCase()),
  );

  const showCustomOption =
    search.trim() &&
    !models.some((m) => m.toLowerCase() === search.toLowerCase()) &&
    !filtered.some((m) => m === search.trim());

  return (
    <div ref={containerRef} className="relative">
      <div className="flex gap-2">
        <div className="relative flex-1">
          {inputMode ? (
            <input
              type="text"
              value={value}
              onChange={(e) => onChange(e.target.value)}
              placeholder="输入自定义模型名称..."
              className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
            />
          ) : (
            <button
              type="button"
              onClick={() => setOpen(!open)}
              className="flex w-full items-center justify-between rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none focus:border-primary transition-colors"
            >
              <span className={cn(!value && "text-muted-foreground/60")}>
                {value || "选择模型..."}
              </span>
              <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform", open && "rotate-180")} />
            </button>
          )}
        </div>
        <button
          type="button"
          onClick={() => setInputMode(!inputMode)}
          className="shrink-0 rounded-lg border border-border px-3 py-2 text-xs font-medium text-muted-foreground transition-all hover:bg-secondary hover:text-foreground active:scale-[0.97]"
          title={inputMode ? "从列表选择" : "手动输入"}
        >
          {inputMode ? "列表" : "手动"}
        </button>
        {loading && (
          <div className="flex items-center">
            <RefreshCw className="h-3.5 w-3.5 animate-spin text-primary/60" />
          </div>
        )}
      </div>

      {!inputMode && open && (
        <div className="animate-fade-in absolute z-50 mt-1.5 max-h-64 w-full overflow-hidden rounded-xl border border-border bg-card shadow-2xl">
          <div className="border-b border-border p-2.5">
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="搜索模型..."
              className="w-full rounded-lg border border-input bg-background px-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
              autoFocus
            />
          </div>
          <div className="max-h-48 overflow-y-auto">
            {filtered.length === 0 && !showCustomOption && (
              <div className="px-4 py-3 text-sm text-muted-foreground">
                {models.length === 0 ? "点击「获取」拉取模型列表" : "无匹配模型"}
              </div>
            )}
            {showCustomOption && (
              <button
                type="button"
                onClick={() => {
                  onChange(search.trim());
                  setOpen(false);
                  setSearch("");
                }}
                className="flex w-full items-center gap-2 px-4 py-2.5 text-left text-sm text-primary transition-colors hover:bg-secondary"
              >
                <ChevronRight className="h-3.5 w-3.5" />
                使用「{search.trim()}」
              </button>
            )}
            {filtered.map((m) => (
              <button
                key={m}
                type="button"
                onClick={() => {
                  onChange(m);
                  setOpen(false);
                  setSearch("");
                }}
                className={cn(
                  "flex w-full items-center px-4 py-2.5 text-left text-sm transition-colors hover:bg-secondary",
                  m === value && "font-medium text-primary",
                )}
              >
                {m}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

const SettingsPage = () => {
  const [config, setConfig] = useState<AiConfig>({
    api_key: "",
    model: "",
    base_url: "https://api.openai.com/v1",
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [models, setModels] = useState<string[]>([]);
  const [modelsLoading, setModelsLoading] = useState(false);

  const handleFetchModels = async (apiKey?: string, baseUrl?: string, currentModel?: string) => {
    const key = apiKey ?? config.api_key;
    const url = baseUrl ?? config.base_url;
    if (!key.trim()) return;
    setModelsLoading(true);
    try {
      const list = await aiApi.listModels(key, url);
      setModels(list);
      if (!currentModel && list.length > 0) {
        setConfig((prev) => ({ ...prev, model: list[0] }));
      }
    } catch (err) {
      console.error("Failed to fetch models:", err);
    } finally {
      setModelsLoading(false);
    }
  };

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const data = await aiApi.getConfig();
        setConfig(data);
        if (data.api_key.trim()) {
          handleFetchModels(data.api_key, data.base_url, data.model);
        }
      } catch (err) {
        console.error("Failed to load config:", err);
      } finally {
        setLoading(false);
      }
    };
    loadConfig();
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setSaved(false);
    try {
      await aiApi.setConfig(config.api_key, config.model, config.base_url);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save config:", err);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex flex-col items-center gap-3">
          <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">加载中...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border px-6 py-4">
        <div className="flex items-center gap-2.5">
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/15">
            <Settings className="h-4 w-4 text-primary" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-foreground">设置</h1>
            <p className="text-sm text-muted-foreground">配置 AI 模型和 API 连接</p>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-6">
        <div className="mx-auto max-w-2xl space-y-6">
          <div className="rounded-xl border border-border bg-card p-6">
            <div className="mb-5 flex items-center gap-2.5">
              <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-primary/12">
                <Zap className="h-4 w-4 text-primary/80" />
              </div>
              <div>
                <h2 className="text-base font-semibold text-foreground">AI 配置</h2>
                <p className="text-xs text-muted-foreground">连接 OpenAI 兼容的 API 服务</p>
              </div>
            </div>

            <div className="space-y-5">
              <div>
                <label className="mb-2 flex items-center gap-2 text-sm font-medium text-foreground">
                  <Globe className="h-3.5 w-3.5 text-muted-foreground" />
                  API Base URL
                </label>
                <input
                  type="text"
                  value={config.base_url}
                  onChange={(e) =>
                    setConfig((prev) => ({
                      ...prev,
                      base_url: e.target.value,
                    }))
                  }
                  placeholder="https://api.openai.com/v1"
                  className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
                />
                <p className="mt-1.5 text-xs text-muted-foreground/70">
                  支持所有 OpenAI 兼容 API 格式的服务地址
                </p>
              </div>

              <div>
                <label className="mb-2 flex items-center gap-2 text-sm font-medium text-foreground">
                  <Key className="h-3.5 w-3.5 text-muted-foreground" />
                  API Key
                </label>
                <input
                  type="password"
                  value={config.api_key}
                  onChange={(e) =>
                    setConfig((prev) => ({ ...prev, api_key: e.target.value }))
                  }
                  placeholder="sk-..."
                  className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
                />
              </div>

              <div>
                <label className="mb-2 flex items-center gap-2 text-sm font-medium text-foreground">
                  <Cpu className="h-3.5 w-3.5 text-muted-foreground" />
                  模型
                </label>
                <ModelCombobox
                  value={config.model}
                  onChange={(val) =>
                    setConfig((prev) => ({ ...prev, model: val }))
                  }
                  models={models}
                  loading={modelsLoading}
                />
              </div>

              <div className="pt-2">
                <button
                  onClick={handleSave}
                  disabled={saving || !config.api_key.trim() || !config.model.trim()}
                  className={cn(
                    "flex items-center gap-2 rounded-lg px-5 py-2.5 text-sm font-medium text-primary-foreground transition-all",
                    "bg-primary hover:bg-primary/90 active:scale-[0.97] disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100",
                  )}
                >
                  {saved ? (
                    <>
                      <Check className="h-4 w-4" />
                      已保存
                    </>
                  ) : (
                    <>
                      <Save className="h-4 w-4" />
                      {saving ? "保存中..." : "保存配置"}
                    </>
                  )}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
