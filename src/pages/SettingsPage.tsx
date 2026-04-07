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
  Bot,
  Workflow,
  Eye,
  EyeOff,
} from "lucide-react";
import { aiApi, workflowApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { AiConfig, AiAgent, Workflow as WorkflowType, WorkflowStep, WorkflowStepType, WorkflowStepTypeOption } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Spinner } from "@/components/ui/spinner";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Popover, PopoverTrigger, PopoverContent } from "@/components/ui/popover";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useResource } from "@/hooks/useResource";
import { useDialog } from "@/hooks/useDialog";

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
}) => (
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
        <Button variant="ghost" size="icon-sm" onClick={() => onEdit(model)} className="text-muted-foreground/50 hover:text-foreground">
          <Pencil className="size-3.5" />
        </Button>
        <Button variant="ghost" size="icon-sm" onClick={() => onDelete(model.id)} className="text-muted-foreground/50 hover:text-destructive">
          <Trash2 className="size-3.5" />
        </Button>
      </div>
    </div>
  </Card>
);

const AgentCard = ({
  agent,
  onEdit,
  onDelete,
  onSetDefault,
}: {
  agent: AiAgent;
  onEdit: (a: AiAgent) => void;
  onDelete: (id: string) => void;
  onSetDefault: (id: string) => void;
}) => (
  <Card className={cn("transition-all duration-150", agent.is_default && "ring-primary/30 border-primary/20")}>
    <div className="flex items-center gap-3 px-4 py-3.5">
      <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-cyan-500/12">
        <Bot className="size-4 text-cyan-500/80" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <p className="text-sm font-medium text-card-foreground truncate">{agent.name}</p>
          {agent.is_default && (
            <span className="shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium bg-primary/15 text-primary">默认</span>
          )}
        </div>
        <p className="mt-0.5 text-xs text-muted-foreground/60 truncate">
          模型: {agent.model_name ?? (agent.model_id ? "未知" : "使用默认模型")} · temperature: {agent.temperature}
        </p>
        {agent.system_prompt && (
          <p className="mt-0.5 text-[11px] text-muted-foreground/40 truncate">{agent.system_prompt}</p>
        )}
      </div>
      <div className="flex items-center gap-1">
        {!agent.is_default && (
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={() => onSetDefault(agent.id)}
            className="text-muted-foreground/50 hover:text-amber-500"
          >
            <Star className="size-3.5" />
          </Button>
        )}
        <Button variant="ghost" size="icon-sm" onClick={() => onEdit(agent)} className="text-muted-foreground/50 hover:text-foreground">
          <Pencil className="size-3.5" />
        </Button>
        <Button variant="ghost" size="icon-sm" onClick={() => onDelete(agent.id)} className="text-muted-foreground/50 hover:text-destructive">
          <Trash2 className="size-3.5" />
        </Button>
      </div>
    </div>
  </Card>
);

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
  const [apiKey, setApiKey] = useState("");
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
        <ModelCombobox value={model} onChange={setModel} models={models} loading={modelsLoading} />
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

const AgentForm = ({
  initial,
  models,
  onSave,
  onCancel,
}: {
  initial?: AiAgent | null;
  models: AiConfig[];
  onSave: (data: { name: string; modelId: string | null; systemPrompt: string; temperature?: number }) => void;
  onCancel: () => void;
}) => {
  const [name, setName] = useState(initial?.name ?? "");
  const [modelId, setModelId] = useState<string | null>(initial?.model_id ?? null);
  const [systemPrompt, setSystemPrompt] = useState(initial?.system_prompt ?? "");
  const [temperature, setTemperature] = useState(initial?.temperature ?? 0.8);
  const [saving, setSaving] = useState(false);
  const isEdit = !!initial;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    setSaving(true);
    try {
      await onSave({ name: name.trim(), modelId: modelId || null, systemPrompt: systemPrompt.trim(), temperature });
    } finally {
      setSaving(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <div className="flex flex-col gap-1.5">
        <Label>助手名称</Label>
        <Input
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="如：写作助手、情节顾问"
          autoFocus
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>关联模型</Label>
        <Select value={modelId ?? "__default__"} onValueChange={(v) => setModelId(v === "__default__" ? null : v)}>
          <SelectTrigger className="w-full">
            <SelectValue placeholder="选择模型..." />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="__default__">使用默认模型</SelectItem>
            {models.map((m) => (
              <SelectItem key={m.id} value={m.id}>
                {m.name || m.model}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {models.length === 0 && (
          <p className="text-xs text-muted-foreground/50">请先在「模型」标签页添加模型配置</p>
        )}
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>Temperature</Label>
        <Input
          type="number"
          min={0}
          max={2}
          step={0.1}
          value={temperature}
          onChange={(e) => setTemperature(parseFloat(e.target.value) || 0)}
          placeholder="0.8"
        />
      </div>
      <div className="flex flex-col gap-1.5">
        <Label>System Prompt</Label>
        <Textarea
          value={systemPrompt}
          onChange={(e) => setSystemPrompt(e.target.value)}
          placeholder="定义助手的角色、行为和规则..."
          rows={6}
          className="resize-y"
        />
      </div>
      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>取消</Button>
        <Button type="submit" disabled={saving || !name.trim()}>
          {saving ? "保存中..." : isEdit ? "更新" : "添加"}
        </Button>
      </DialogFooter>
    </form>
  );
};

const STEP_TYPE_OPTIONS: WorkflowStepTypeOption[] = [
  { value: "generate_worldview", label: "世界观生成" },
  { value: "generate_characters", label: "角色生成" },
  { value: "generate_volume_structure", label: "卷结构生成" },
  { value: "generate_chapter_structure", label: "章节结构生成" },
  { value: "expand_chapter_outline", label: "章节扩写" },
  { value: "narrate", label: "推进叙事" },
  { value: "character_action", label: "角色行动" },
  { value: "polish", label: "润色" },
  { value: "rewrite", label: "改写" },
  { value: "continue_writing", label: "续写" },
  { value: "dialogue", label: "对话生成" },
  { value: "review", label: "质量审查" },
];

const DEFAULT_AGENT: Record<string, string> = {
  narrate: "叙事者",
  character_action: "角色扮演",
  review: "质量审查",
  polish: "润色助手",
  rewrite: "改写助手",
  continue_writing: "续写助手",
  dialogue: "对话生成",
  generate_worldview: "大纲生成",
  generate_characters: "大纲生成",
  generate_volume_structure: "大纲生成",
  generate_chapter_structure: "大纲生成",
  expand_chapter_outline: "大纲生成",
};

const WorkflowCard = ({
  workflow,
  onEdit,
  onDelete,
  onSetDefault,
}: {
  workflow: WorkflowType;
  onEdit: () => void;
  onDelete: (id: string) => void;
  onSetDefault: (id: string) => void;
}) => (
  <Card className={cn("transition-all duration-150", workflow.is_default && "ring-primary/30 border-primary/20")}>
    <div className="flex items-center gap-3 px-4 py-3.5">
      <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-violet-500/12">
        <Workflow className="size-4 text-violet-500/80" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <p className="text-sm font-medium text-card-foreground truncate">{workflow.name}</p>
          {workflow.is_default && (
            <span className="shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium bg-primary/15 text-primary">默认</span>
          )}
          {workflow.is_preset && (
            <span className="shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium bg-muted text-muted-foreground">预设</span>
          )}
        </div>
        <p className="mt-0.5 text-xs text-muted-foreground/60 truncate">{workflow.description}</p>
        <p className="mt-0.5 text-[11px] text-muted-foreground/40">{workflow.step_count} 个步骤</p>
      </div>
      <div className="flex items-center gap-1">
        <Button variant="ghost" size="icon-sm" onClick={onEdit} className="text-muted-foreground/50 hover:text-foreground">
          <Pencil className="size-3.5" />
        </Button>
        {!workflow.is_preset && (
          <>
            {!workflow.is_default && (
              <Button variant="ghost" size="icon-sm" onClick={() => onSetDefault(workflow.id)} className="text-muted-foreground/50 hover:text-amber-500">
                <Star className="size-3.5" />
              </Button>
            )}
            <Button variant="ghost" size="icon-sm" onClick={() => onDelete(workflow.id)} className="text-muted-foreground/50 hover:text-destructive">
              <Trash2 className="size-3.5" />
            </Button>
          </>
        )}
      </div>
    </div>
  </Card>
);

interface StepRow {
  step_type: WorkflowStepType;
  agent_id: string;
  condition: string;
  enabled: boolean;
}

type WorkflowWithSteps = WorkflowType & { steps?: WorkflowStep[] };

const WorkflowForm = ({
  initial,
  agents,
  onSave,
  onCancel,
}: {
  initial?: WorkflowWithSteps;
  agents: AiAgent[];
  onSave: (data: { name: string; description: string; steps: Array<{ step_type: WorkflowStepType; agent_id: string | null; condition: Record<string, unknown> | null; config: Record<string, unknown>; enabled: boolean }> }) => Promise<void>;
  onCancel: () => void;
}) => {
  const isEdit = !!initial;
  const [saving, setSaving] = useState(false);
  const [name, setName] = useState(initial?.name ?? "");
  const [description, setDescription] = useState(initial?.description ?? "");
  const [steps, setSteps] = useState<StepRow[]>(
    initial?.steps?.map((s) => ({
      step_type: s.step_type as WorkflowStepType,
      agent_id: s.agent_id ?? "",
      condition: s.condition ? JSON.stringify(s.condition) : "",
      enabled: s.enabled,
    })) ?? [{ step_type: "narrate", agent_id: "", condition: "", enabled: true }],
  );

  const addStep = () => {
    setSteps((prev) => [...prev, { step_type: "narrate", agent_id: "", condition: "", enabled: true }]);
  };

  const removeStep = (idx: number) => {
    setSteps((prev) => prev.filter((_, i) => i !== idx));
  };

  const updateStep = (idx: number, patch: Partial<StepRow>) => {
    setSteps((prev) => prev.map((s, i) => (i === idx ? { ...s, ...patch } : s)));
  };

  const moveStep = (idx: number, dir: -1 | 1) => {
    const next = idx + dir;
    if (next < 0 || next >= steps.length) return;
    setSteps((prev) => {
      const arr = [...prev];
      [arr[idx], arr[next]] = [arr[next], arr[idx]];
      return arr;
    });
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    try {
      await onSave({ name, description, steps: steps.map((s) => ({
        step_type: s.step_type,
        agent_id: s.agent_id || null,
        condition: s.condition.trim() ? JSON.parse(s.condition) as Record<string, unknown> : null,
        config: {} as Record<string, unknown>,
        enabled: s.enabled,
      }))});
    } finally {
      setSaving(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <div>
        <Label htmlFor="wf-name">名称</Label>
        <Input id="wf-name" value={name} onChange={(e) => setName(e.target.value)} placeholder="工作流名称" />
      </div>
      <div>
        <Label htmlFor="wf-desc">描述</Label>
        <Input id="wf-desc" value={description} onChange={(e) => setDescription(e.target.value)} placeholder="简要描述" />
      </div>
      <div>
        <Label>步骤</Label>
        <div className="flex flex-col gap-2 mt-1">
          {steps.map((step, idx) => (
            <div key={idx} className="flex items-center gap-2 rounded-lg border bg-muted/30 p-2">
              <div className="flex flex-col gap-0.5">
                <button type="button" className="text-muted-foreground/40 hover:text-foreground disabled:opacity-20" disabled={idx === 0} onClick={() => moveStep(idx, -1)}>
                  <ChevronDown className="size-3.5 -rotate-90" />
                </button>
                <button type="button" className="text-muted-foreground/40 hover:text-foreground disabled:opacity-20" disabled={idx === steps.length - 1} onClick={() => moveStep(idx, 1)}>
                  <ChevronDown className="size-3.5 rotate-90" />
                </button>
              </div>
              <div className="flex-1 grid grid-cols-[1fr_1fr] gap-2">
                <Select value={step.step_type} onValueChange={(v) => {
                  const defAgent = DEFAULT_AGENT[v as string];
                  updateStep(idx, { step_type: v as WorkflowStepType, agent_id: defAgent ? (agents.find((a) => a.name === defAgent)?.id ?? step.agent_id) : step.agent_id });
                }}>
                  <SelectTrigger className="h-7 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {STEP_TYPE_OPTIONS.map((opt) => (
                      <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Select value={step.agent_id || ""} onValueChange={(v) => updateStep(idx, { agent_id: v ?? "" })}>
                  <SelectTrigger className="h-7 text-xs">
                    <SelectValue placeholder="使用默认 Agent" />
                  </SelectTrigger>
                  <SelectContent>
                    {agents.filter((a) => a.id != null).map((a) => (
                      <SelectItem key={a.id} value={a.id!}>{a.name}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <button type="button" className="text-muted-foreground/50 hover:text-foreground" onClick={() => updateStep(idx, { enabled: !step.enabled })}>
                {step.enabled ? <Eye className="size-3.5" /> : <EyeOff className="size-3.5" />}
              </button>
              <button type="button" className="text-muted-foreground/50 hover:text-destructive" onClick={() => removeStep(idx)}>
                <Trash2 className="size-3.5" />
              </button>
            </div>
          ))}
          <Button type="button" variant="outline" size="sm" className="w-full" onClick={addStep}>
            <Plus className="size-3.5" />
            添加步骤
          </Button>
        </div>
      </div>
      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>取消</Button>
        <Button type="submit" disabled={saving || !name.trim()}>
          {saving ? "保存中..." : isEdit ? "更新" : "添加"}
        </Button>
      </DialogFooter>
    </form>
  );
};

const SettingsPage = () => {
  const navigate = useNavigate();
  const { items: models, loading: modelsLoading, reload: reloadModels, remove: removeModel, setItems: setModels } = useResource(aiApi.listModels);
  const { items: agents, loading: agentsLoading, reload: reloadAgents, remove: removeAgent, setItems: setAgents } = useResource(aiApi.listAgents);
  const { items: workflows, loading: workflowsLoading, reload: reloadWorkflows, remove: removeWorkflow, setItems: setWorkflows } = useResource(workflowApi.list);
  const loading = modelsLoading || agentsLoading || workflowsLoading;

  const workflowDialog = useDialog<WorkflowWithSteps>();

  const handleEditWorkflow = (wf: WorkflowType) => {
    workflowApi.getSteps(wf.id).then((steps) => {
      workflowDialog.show({ ...wf, steps });
    });
  };

  const handleDeleteWorkflow = async (id: string) => {
    await workflowApi.delete(id);
    removeWorkflow(id);
  };

  const handleSetDefaultWorkflow = async (id: string) => {
    await workflowApi.setDefault(id);
    setWorkflows((prev) => prev.map((w) => ({ ...w, is_default: w.id === id })));
  };

  const handleSaveWorkflow = async (data: { name: string; description: string; steps: Array<{ step_type: WorkflowStepType; agent_id: string | null; condition: Record<string, unknown> | null; config: Record<string, unknown>; enabled: boolean }> }) => {
    if (workflowDialog.editing) {
      await workflowApi.update(workflowDialog.editing.id, data.name, data.description, data.steps);
    } else {
      await workflowApi.create(data.name, data.description, data.steps);
    }
    await reloadWorkflows();
    workflowDialog.close();
  };

  const modelDialog = useDialog<AiConfig>();
  const agentDialog = useDialog<AiAgent>();

  const handleCreateModel = async (data: { name: string; apiKey: string; model: string; baseUrl: string }) => {
    await aiApi.createModel(data.name, data.apiKey, data.model, data.baseUrl);
    await reloadModels();
    modelDialog.close();
  };

  const handleUpdateModel = async (data: { name: string; apiKey: string; model: string; baseUrl: string }) => {
    if (!modelDialog.editing) return;
    await aiApi.updateModel(modelDialog.editing.id, data.name, data.apiKey, data.model, data.baseUrl);
    await reloadModels();
    modelDialog.close();
  };

  const handleDeleteModel = async (id: string) => {
    await aiApi.deleteModel(id);
    removeModel(id);
  };

  const handleSetDefaultModel = async (id: string) => {
    await aiApi.setDefault(id);
    setModels((prev) => prev.map((m) => ({ ...m, is_default: m.id === id })));
  };

  const handleCreateAgent = async (data: { name: string; modelId: string | null; systemPrompt: string; temperature?: number }) => {
    await aiApi.createAgent(data.name, data.modelId, data.systemPrompt, data.temperature);
    await reloadAgents();
    agentDialog.close();
  };

  const handleUpdateAgent = async (data: { name: string; modelId: string | null; systemPrompt: string; temperature?: number }) => {
    if (!agentDialog.editing) return;
    await aiApi.updateAgent(agentDialog.editing.id, data.name, data.modelId, data.systemPrompt, data.temperature);
    await reloadAgents();
    agentDialog.close();
  };

  const handleDeleteAgent = async (id: string) => {
    await aiApi.deleteAgent(id);
    removeAgent(id);
  };

  const handleSetDefaultAgent = async (id: string) => {
    await aiApi.setDefaultAgent(id);
    setAgents((prev) => prev.map((a) => ({ ...a, is_default: a.id === id })));
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
          <Tabs defaultValue="models">
            <TabsList>
              <TabsTrigger value="models">模型</TabsTrigger>
              <TabsTrigger value="agents">助手</TabsTrigger>
              <TabsTrigger value="workflows">工作流</TabsTrigger>
            </TabsList>

            <TabsContent value="models">
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2.5">
                      <div className="flex size-8 items-center justify-center rounded-lg bg-primary/12">
                        <Zap className="size-4 text-primary/80" />
                      </div>
                      <div>
                        <CardTitle>AI 模型配置</CardTitle>
                        <CardDescription>管理 API 连接和模型</CardDescription>
                      </div>
                    </div>
                    <Button onClick={() => modelDialog.show()} data-icon="inline-start">
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
                      <Button variant="outline" size="sm" onClick={() => modelDialog.show()} data-icon="inline-start">
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
                          onEdit={(model) => modelDialog.show(model)}
                          onDelete={handleDeleteModel}
                          onSetDefault={handleSetDefaultModel}
                        />
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="agents">
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2.5">
                      <div className="flex size-8 items-center justify-center rounded-lg bg-cyan-500/12">
                        <Bot className="size-4 text-cyan-500/80" />
                      </div>
                      <div>
                        <CardTitle>AI 助手</CardTitle>
                        <CardDescription>配置不同角色的 AI 助手</CardDescription>
                      </div>
                    </div>
                    <Button onClick={() => agentDialog.show()} data-icon="inline-start">
                      <Plus />
                      添加助手
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {loading ? (
                    <div className="flex h-20 items-center justify-center">
                      <Spinner className="size-5 text-primary" />
                    </div>
                  ) : agents.length === 0 ? (
                    <div className="flex flex-col items-center gap-2 py-8 text-muted-foreground/60">
                      <Bot className="size-8 text-muted-foreground/30" />
                      <p className="text-sm">还没有配置助手</p>
                      <Button variant="outline" size="sm" onClick={() => agentDialog.show()} data-icon="inline-start">
                        <Plus />
                        添加助手
                      </Button>
                    </div>
                  ) : (
                    <div className="flex flex-col gap-2">
                      {agents.map((a) => (
                        <AgentCard
                          key={a.id}
                          agent={a}
                          onEdit={(agent) => agentDialog.show(agent)}
                          onDelete={handleDeleteAgent}
                          onSetDefault={handleSetDefaultAgent}
                        />
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="workflows">
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle>工作流</CardTitle>
                      <CardDescription>配置 AI 工作流管线，定义步骤顺序和条件</CardDescription>
                    </div>
                    <Button onClick={() => workflowDialog.show()} data-icon="inline-start">
                      <Plus />
                      添加工作流
                    </Button>
                  </div>
                </CardHeader>
                <CardContent>
                  {loading ? (
                    <div className="flex h-20 items-center justify-center">
                      <Spinner className="size-5 text-primary" />
                    </div>
                  ) : workflows.length === 0 ? (
                    <div className="flex flex-col items-center gap-2 py-8 text-muted-foreground/60">
                      <Workflow className="size-8 text-muted-foreground/30" />
                      <p className="text-sm">还没有配置工作流</p>
                    </div>
                  ) : (
                    <div className="flex flex-col gap-2">
                      {workflows.map((wf) => (
                        <WorkflowCard
                          key={wf.id}
                          workflow={wf}
                          onEdit={() => handleEditWorkflow(wf)}
                          onDelete={() => handleDeleteWorkflow(wf.id)}
                          onSetDefault={() => handleSetDefaultWorkflow(wf.id)}
                        />
                      ))}
                    </div>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      </div>

      <Dialog open={modelDialog.open} onOpenChange={modelDialog.onOpenChange}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{modelDialog.isEditing ? "编辑模型" : "添加模型"}</DialogTitle>
            <DialogDescription>{modelDialog.isEditing ? "修改 AI 模型配置" : "添加一个新的 AI 模型配置"}</DialogDescription>
          </DialogHeader>
          <ModelForm
            key={modelDialog.editing?.id ?? "new"}
            initial={modelDialog.editing}
            onSave={modelDialog.isEditing ? handleUpdateModel : handleCreateModel}
            onCancel={modelDialog.close}
          />
        </DialogContent>
      </Dialog>

      <Dialog open={agentDialog.open} onOpenChange={agentDialog.onOpenChange}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>{agentDialog.isEditing ? "编辑助手" : "添加助手"}</DialogTitle>
            <DialogDescription>{agentDialog.isEditing ? "修改 AI 助手配置" : "添加一个新的 AI 助手"}</DialogDescription>
          </DialogHeader>
          <AgentForm
            key={agentDialog.editing?.id ?? "new"}
            initial={agentDialog.editing}
            models={models}
            onSave={agentDialog.isEditing ? handleUpdateAgent : handleCreateAgent}
            onCancel={agentDialog.close}
          />
        </DialogContent>
      </Dialog>

      <Dialog open={workflowDialog.open} onOpenChange={workflowDialog.onOpenChange}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>{workflowDialog.isEditing ? "编辑工作流" : "添加工作流"}</DialogTitle>
            <DialogDescription>{workflowDialog.isEditing ? "修改工作流步骤和配置" : "创建一个新的 AI 工作流"}</DialogDescription>
          </DialogHeader>
          <WorkflowForm
            key={workflowDialog.editing?.id ?? "new"}
            initial={workflowDialog.editing ?? undefined}
            agents={agents}
            onSave={handleSaveWorkflow}
            onCancel={workflowDialog.close}
          />
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default SettingsPage;
