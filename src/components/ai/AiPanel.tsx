import { useState, useEffect } from "react";
import {
  Sparkles,
  PenLine,
  RefreshCw,
  MessageSquare,
  MessagesSquare,
  Send,
  Copy,
  Check,
  X,
  ArrowDownToLine,
  Square,
} from "lucide-react";

import { aiApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Empty, EmptyHeader, EmptyMedia, EmptyDescription } from "@/components/ui/empty";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Select, SelectTrigger, SelectContent, SelectGroup, SelectItem, SelectValue } from "@/components/ui/select";
import { useAiEditor } from "@/contexts/AiEditorContext";
import type { AiMode } from "@/lib/types";

interface AiPanelProps {
  open: boolean;
  onClose: () => void;
}

const modes: { key: AiMode; label: string; icon: typeof Sparkles }[] = [
  { key: "continue", label: "续写", icon: PenLine },
  { key: "rewrite", label: "改写", icon: RefreshCw },
  { key: "polish", label: "润色", icon: Sparkles },
  { key: "dialogue", label: "对话", icon: MessageSquare },
  { key: "chat", label: "聊天", icon: MessagesSquare },
];

const styleOptions = [
  { value: "文学小说", label: "文学小说" },
  { value: "悬疑推理", label: "悬疑推理" },
  { value: "奇幻冒险", label: "奇幻冒险" },
  { value: "都市情感", label: "都市情感" },
  { value: "历史传记", label: "历史传记" },
];

const lengthOptions = [
  { value: "short", label: "短 (100-200字)" },
  { value: "medium", label: "中 (300-500字)" },
  { value: "long", label: "长 (600-1000字)" },
];

const ContinuePanel = () => {
  const { editorState, startStreaming, isStreaming } = useAiEditor();
  const [style, setStyle] = useState("文学小说");
  const [length, setLength] = useState("medium");

  const handleGenerate = () => {
    const context = editorState.cursorBefore || "";
    if (!context.trim() && !editorState.hasSelection) return;
    startStreaming({
      mode: "continue",
      text: editorState.selectedText || context,
      style,
      length,
    });
  };

  const canGenerate =
    (editorState.cursorBefore || editorState.selectedText).trim().length > 0;

  return (
    <div className="flex flex-col gap-3">
      {editorState.cursorBefore && (
        <div className="rounded-lg border border-border bg-muted/30 p-3 text-xs text-muted-foreground max-h-24 overflow-y-auto">
          <div className="mb-1 font-medium text-foreground/60">当前上下文</div>
          <div className="whitespace-pre-wrap line-clamp-4">
            {editorState.cursorBefore.slice(-200)}
            {editorState.cursorBefore.length > 200 && "..."}
          </div>
        </div>
      )}
      <div className="flex gap-2">
        <div className="flex flex-col gap-1.5 flex-1">
          <Label className="text-xs">风格</Label>
          <Select value={style} onValueChange={(v) => { if (v) setStyle(v); }}>
            <SelectTrigger size="sm" className="w-full">
              <SelectValue>{styleOptions.find(o => o.value === style)?.label}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {styleOptions.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col gap-1.5 flex-1">
          <Label className="text-xs">长度</Label>
          <Select value={length} onValueChange={(v) => { if (v) setLength(v); }}>
            <SelectTrigger size="sm" className="w-full">
              <SelectValue>{lengthOptions.find(o => o.value === length)?.label}</SelectValue>
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {lengthOptions.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
      </div>
      <Button
        onClick={handleGenerate}
        disabled={isStreaming || !canGenerate}
        className="w-full"
      >
        {isStreaming ? "生成中..." : "续写"}
      </Button>
    </div>
  );
};

const RewritePanel = () => {
  const { editorState, startStreaming, isStreaming } = useAiEditor();
  const [instruction, setInstruction] = useState("");
  const [localText, setLocalText] = useState("");

  const text = editorState.selectedText || localText;

  const handleRewrite = () => {
    if (!text.trim()) return;
    startStreaming({
      mode: "rewrite",
      text,
      instruction: instruction || undefined,
    });
  };

  return (
    <div className="flex flex-col gap-3">
      {editorState.hasSelection ? (
        <div className="rounded-lg border border-primary/30 bg-primary/5 p-3 text-sm">
          <div className="mb-1 text-xs text-primary font-medium">选中的文字</div>
          <div className="whitespace-pre-wrap max-h-32 overflow-y-auto">
            {editorState.selectedText.slice(0, 300)}
            {editorState.selectedText.length > 300 && "..."}
          </div>
        </div>
      ) : (
        <Textarea
          value={localText}
          onChange={(e) => setLocalText(e.target.value)}
          placeholder="输入要改写的文字..."
          rows={4}
        />
      )}
      <Input
        value={instruction}
        onChange={(e) => setInstruction(e.target.value)}
        placeholder="改写指令（如：更正式、更活泼...）"
      />
      <Button
        onClick={handleRewrite}
        disabled={isStreaming || !text.trim()}
        className="w-full"
      >
        {isStreaming ? "改写中..." : "改写"}
      </Button>
    </div>
  );
};

const PolishPanel = () => {
  const { editorState, startStreaming, isStreaming } = useAiEditor();
  const [localText, setLocalText] = useState("");

  const text = editorState.selectedText || localText;

  const handlePolish = () => {
    if (!text.trim()) return;
    startStreaming({ mode: "polish", text });
  };

  return (
    <div className="flex flex-col gap-3">
      {editorState.hasSelection ? (
        <div className="rounded-lg border border-primary/30 bg-primary/5 p-3 text-sm">
          <div className="mb-1 text-xs text-primary font-medium">选中的文字</div>
          <div className="whitespace-pre-wrap max-h-32 overflow-y-auto">
            {editorState.selectedText.slice(0, 300)}
            {editorState.selectedText.length > 300 && "..."}
          </div>
        </div>
      ) : (
        <Textarea
          value={localText}
          onChange={(e) => setLocalText(e.target.value)}
          placeholder="输入要润色的文字..."
          rows={4}
        />
      )}
      <Button
        onClick={handlePolish}
        disabled={isStreaming || !text.trim()}
        className="w-full"
      >
        {isStreaming ? "润色中..." : "润色"}
      </Button>
    </div>
  );
};

const DialoguePanel = () => {
  const { startStreaming, isStreaming } = useAiEditor();
  const [characters, setCharacters] = useState("");
  const [scenario, setScenario] = useState("");

  const handleGenerate = () => {
    if (!characters.trim() || !scenario.trim()) return;
    const text = `角色信息：\n${characters}\n\n场景描述：\n${scenario}`;
    startStreaming({ mode: "dialogue", text });
  };

  return (
    <div className="flex flex-col gap-3">
      <Textarea
        value={characters}
        onChange={(e) => setCharacters(e.target.value)}
        placeholder="描述参与对话的角色（名字、性格、关系）..."
        rows={3}
      />
      <Textarea
        value={scenario}
        onChange={(e) => setScenario(e.target.value)}
        placeholder="描述对话发生的场景..."
        rows={2}
      />
      <Button
        onClick={handleGenerate}
        disabled={isStreaming || !characters.trim() || !scenario.trim()}
        className="w-full"
      >
        {isStreaming ? "生成中..." : "生成对话"}
      </Button>
    </div>
  );
};

const ChatPanel = () => {
  const { editorState, startStreaming, isStreaming, streamingText } = useAiEditor();
  const [messages, setMessages] = useState<Array<{ role: string; content: string }>>([]);
  const [input, setInput] = useState("");
  const [initialLoad, setInitialLoad] = useState(true);

  useEffect(() => {
    if (initialLoad && editorState.projectId) {
      aiApi.getChatHistory(editorState.projectId).then((history) => {
        setMessages(history);
        setInitialLoad(false);
      }).catch(() => {
        setInitialLoad(false);
      });
    }
  }, [editorState.projectId, initialLoad]);

  const handleSend = async () => {
    if (!input.trim() || isStreaming) return;
    const userMsg = input.trim();
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: userMsg }]);

    startStreaming({
      mode: "chat",
      text: userMsg,
    });
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex-1 overflow-y-auto flex flex-col gap-3 py-1">
        {messages.length === 0 && !isStreaming && (
          <Empty className="py-12 border-transparent">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <MessagesSquare />
              </EmptyMedia>
              <EmptyDescription>向 AI 助手咨询写作相关问题</EmptyDescription>
            </EmptyHeader>
          </Empty>
        )}
        {messages.map((msg, i) => (
          <div
            key={i}
            className={cn(
              "rounded-xl px-3.5 py-2.5 text-sm leading-relaxed animate-fade-in",
              msg.role === "user"
                ? "bg-primary/12 text-foreground ml-6"
                : "bg-muted/60 text-foreground mr-2",
            )}
          >
            {msg.content}
          </div>
        ))}
        {isStreaming && streamingText && (
          <div className="mr-2 rounded-xl bg-muted/60 px-3.5 py-2.5 text-sm leading-relaxed animate-fade-in">
            {streamingText}
            <span className="inline-block size-1.5 ml-0.5 rounded-full bg-primary/50 animate-pulse" />
          </div>
        )}
        {isStreaming && !streamingText && (
          <div className="mr-2 flex items-center gap-2 rounded-xl bg-muted/60 px-3.5 py-2.5 text-sm text-muted-foreground">
            <Spinner className="size-3 text-primary/50" />
            思考中...
          </div>
        )}
      </div>
      <div className="flex gap-2 pt-3 border-t border-border">
        <Input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              handleSend();
            }
          }}
          placeholder="输入问题..."
          disabled={isStreaming}
        />
        {isStreaming ? (
          <Button size="icon" variant="destructive" onClick={() => {}}>
            <Square className="size-3" />
          </Button>
        ) : (
          <Button onClick={handleSend} disabled={!input.trim()} size="icon">
            <Send />
          </Button>
        )}
      </div>
    </div>
  );
};

const AiPanel = ({ open, onClose }: AiPanelProps) => {
  const {
    activeMode,
    setActiveMode,
    isStreaming,
    streamingText,
    generatedText,
    stopStreaming,
    replaceSelection,
    editorState,
  } = useAiEditor();

  const [copied, setCopied] = useState(false);

  const finalText = generatedText || streamingText;

  if (!open) return null;

  const handleCopy = async () => {
    await navigator.clipboard.writeText(finalText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleInsert = () => {
    if (!finalText) return;
    if (editorState.hasSelection) {
      replaceSelection(finalText);
    } else {
      replaceSelection(finalText);
    }
  };

  return (
    <div className="flex h-full w-[380px] shrink-0 flex-col border-l border-border bg-background animate-fade-in">
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2.5">
          <div className="flex size-7 items-center justify-center rounded-lg bg-primary/15">
            <Sparkles className="size-3.5 text-primary" />
          </div>
          <span className="text-sm font-semibold text-foreground">AI 助手</span>
          {isStreaming && (
            <span className="flex items-center gap-1.5 text-xs text-primary/70">
              <Spinner className="size-2.5" />
              输出中
            </span>
          )}
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={onClose}
        >
          <X />
        </Button>
      </div>

      <Tabs
        value={activeMode}
        onValueChange={(v) => setActiveMode(v as AiMode)}
        className="flex flex-col flex-1 overflow-hidden"
      >
        <div className="border-b border-border px-3 py-2">
          <TabsList variant="line">
            {modes.map((mode) => (
              <TabsTrigger key={mode.key} value={mode.key} data-icon="inline-start">
                <mode.icon />
                {mode.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </div>

        <div className="flex-1 overflow-y-auto p-4">
          <TabsContent value="continue">
            <ContinuePanel />
          </TabsContent>
          <TabsContent value="rewrite">
            <RewritePanel />
          </TabsContent>
          <TabsContent value="polish">
            <PolishPanel />
          </TabsContent>
          <TabsContent value="dialogue">
            <DialoguePanel />
          </TabsContent>
          <TabsContent value="chat">
            <ChatPanel />
          </TabsContent>
        </div>
      </Tabs>

      {finalText && !isStreaming && (
        <div className="flex gap-2 border-t border-border px-4 py-3">
          <Button
            onClick={handleInsert}
            className="flex-1"
            data-icon="inline-start"
          >
            <ArrowDownToLine />
            {editorState.hasSelection ? "替换选中" : "插入到编辑器"}
          </Button>
          <Button
            variant="outline"
            size="icon"
            onClick={handleCopy}
          >
            {copied ? (
              <Check />
            ) : (
              <Copy />
            )}
          </Button>
        </div>
      )}

      {isStreaming && (
        <div className="flex gap-2 border-t border-border px-4 py-3">
          <Button variant="outline" onClick={stopStreaming} className="flex-1" data-icon="inline-start">
            <Square className="size-3" />
            停止生成
          </Button>
        </div>
      )}

      {activeMode === "chat" || activeMode === "dialogue" ? null : isStreaming && (
        <div className="mx-4 mb-3 rounded-lg border border-border bg-muted/50 p-3 text-sm whitespace-pre-wrap max-h-40 overflow-y-auto">
          {streamingText}
          <span className="inline-block size-1.5 ml-0.5 rounded-full bg-primary/50 animate-pulse" />
        </div>
      )}
    </div>
  );
};

export default AiPanel;
