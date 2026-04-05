import { useState, useEffect } from "react";
import { Key, Cpu, Globe, Save, Check, RefreshCw, ChevronDown, Settings, Zap, ChevronRight } from "lucide-react";
import { aiApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { AiConfig } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Popover, PopoverTrigger, PopoverContent } from "@/components/ui/popover";

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
                    <ChevronRight className="size-3.5" />
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
          <Spinner className="size-5 text-primary" />
          <p className="text-sm text-muted-foreground">加载中...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-border px-6 py-4">
        <div className="flex items-center gap-2.5">
          <div className="flex size-8 items-center justify-center rounded-lg bg-primary/15">
            <Settings className="size-4 text-primary" />
          </div>
          <div>
            <h1 className="text-lg font-semibold text-foreground">设置</h1>
            <p className="text-sm text-muted-foreground">配置 AI 模型和 API 连接</p>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-6">
        <div className="mx-auto max-w-2xl">
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2.5">
                <div className="flex size-8 items-center justify-center rounded-lg bg-primary/12">
                  <Zap className="size-4 text-primary/80" />
                </div>
                <div>
                  <CardTitle>AI 配置</CardTitle>
                  <CardDescription>连接 OpenAI 兼容的 API 服务</CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="flex flex-col gap-5">
              <div className="flex flex-col gap-1.5">
                <Label data-icon="inline-start">
                  <Globe />
                  API Base URL
                </Label>
                <Input
                  type="text"
                  value={config.base_url}
                  onChange={(e) =>
                    setConfig((prev) => ({
                      ...prev,
                      base_url: e.target.value,
                    }))
                  }
                  placeholder="https://api.openai.com/v1"
                />
                <p className="text-xs text-muted-foreground/70">
                  支持所有 OpenAI 兼容 API 格式的服务地址
                </p>
              </div>

              <div className="flex flex-col gap-1.5">
                <Label data-icon="inline-start">
                  <Key />
                  API Key
                </Label>
                <Input
                  type="password"
                  value={config.api_key}
                  onChange={(e) =>
                    setConfig((prev) => ({ ...prev, api_key: e.target.value }))
                  }
                  placeholder="sk-..."
                />
              </div>

              <div className="flex flex-col gap-1.5">
                <Label data-icon="inline-start">
                  <Cpu />
                  模型
                </Label>
                <ModelCombobox
                  value={config.model}
                  onChange={(val) =>
                    setConfig((prev) => ({ ...prev, model: val }))
                  }
                  models={models}
                  loading={modelsLoading}
                />
              </div>

              <div className="flex justify-end pt-2">
                <Button
                  onClick={handleSave}
                  disabled={saving || !config.api_key.trim() || !config.model.trim()}
                  data-icon="inline-start"
                >
                  {saved ? (
                    <>
                      <Check />
                      已保存
                    </>
                  ) : (
                    <>
                      <Save />
                      {saving ? "保存中..." : "保存配置"}
                    </>
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default SettingsPage;
