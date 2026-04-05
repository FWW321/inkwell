import { useState } from "react";
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
} from "lucide-react";

import { aiApi } from "@/lib/api";
import { cn } from "@/lib/utils";

type AiMode = "continue" | "rewrite" | "polish" | "dialogue" | "chat";

interface AiPanelProps {
  open: boolean;
  onClose: () => void;
  selectedText: string;
  onInsert: (text: string) => void;
}

const modes: { key: AiMode; label: string; icon: typeof Sparkles }[] = [
  { key: "continue", label: "续写", icon: PenLine },
  { key: "rewrite", label: "改写", icon: RefreshCw },
  { key: "polish", label: "润色", icon: Sparkles },
  { key: "dialogue", label: "对话", icon: MessageSquare },
  { key: "chat", label: "聊天", icon: MessagesSquare },
];

const ContinuePanel = ({
  onResult,
}: { onResult: (text: string) => void }) => {
  const [context, setContext] = useState("");
  const [style, setStyle] = useState("文学小说");
  const [length, setLength] = useState("medium");
  const [result, setResult] = useState("");
  const [loading, setLoading] = useState(false);

  const handleGenerate = async () => {
    if (!context.trim()) return;
    setLoading(true);
    setResult("");
    try {
      const text = await aiApi.continueWriting(context, style, length);
      setResult(text);
      onResult(text);
    } catch (err) {
      setResult(`错误: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-3">
      <textarea
        value={context}
        onChange={(e) => setContext(e.target.value)}
        placeholder="粘贴或输入前文内容..."
        rows={4}
        className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
      />
      <div className="flex gap-2">
        <select
          value={style}
          onChange={(e) => setStyle(e.target.value)}
          className="flex-1 rounded-lg border border-input bg-background px-2.5 py-1.5 text-sm outline-none cursor-pointer transition-colors"
          style={{ colorScheme: "dark" }}
        >
          <option value="文学小说">文学小说</option>
          <option value="悬疑推理">悬疑推理</option>
          <option value="奇幻冒险">奇幻冒险</option>
          <option value="都市情感">都市情感</option>
          <option value="历史传记">历史传记</option>
        </select>
        <select
          value={length}
          onChange={(e) => setLength(e.target.value)}
          className="flex-1 rounded-lg border border-input bg-background px-2.5 py-1.5 text-sm outline-none cursor-pointer transition-colors"
          style={{ colorScheme: "dark" }}
        >
          <option value="short">短 (100-200字)</option>
          <option value="medium">中 (300-500字)</option>
          <option value="long">长 (600-1000字)</option>
        </select>
      </div>
      <button
        onClick={handleGenerate}
        disabled={loading || !context.trim()}
        className={cn(
          "w-full rounded-lg px-4 py-2.5 text-sm font-medium text-primary-foreground transition-all",
          "bg-primary hover:bg-primary/90 active:scale-[0.98] disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100",
        )}
      >
        {loading ? "生成中..." : "续写"}
      </button>
      {result && (
        <div className="rounded-lg border border-border bg-muted/50 p-3 text-sm whitespace-pre-wrap animate-fade-in">
          {result}
        </div>
      )}
    </div>
  );
};

const RewritePanel = ({
  selectedText,
  onResult,
}: {
  selectedText: string;
  onResult: (text: string) => void;
}) => {
  const [text, setText] = useState(selectedText);
  const [instruction, setInstruction] = useState("");
  const [result, setResult] = useState("");
  const [loading, setLoading] = useState(false);

  const handleRewrite = async () => {
    if (!text.trim()) return;
    setLoading(true);
    setResult("");
    try {
      const res = await aiApi.rewrite(text, instruction || "改写");
      setResult(res);
      onResult(res);
    } catch (err) {
      setResult(`错误: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-3">
      <textarea
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder="输入要改写的文字..."
        rows={3}
        className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
      />
      <input
        value={instruction}
        onChange={(e) => setInstruction(e.target.value)}
        placeholder="改写指令（如：更正式、更活泼...）"
        className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
      />
      <button
        onClick={handleRewrite}
        disabled={loading || !text.trim()}
        className={cn(
          "w-full rounded-lg px-4 py-2.5 text-sm font-medium text-primary-foreground transition-all",
          "bg-primary hover:bg-primary/90 active:scale-[0.98] disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100",
        )}
      >
        {loading ? "改写中..." : "改写"}
      </button>
      {result && (
        <div className="rounded-lg border border-border bg-muted/50 p-3 text-sm whitespace-pre-wrap animate-fade-in">
          {result}
        </div>
      )}
    </div>
  );
};

const PolishPanel = ({
  selectedText,
  onResult,
}: {
  selectedText: string;
  onResult: (text: string) => void;
}) => {
  const [text, setText] = useState(selectedText);
  const [result, setResult] = useState("");
  const [loading, setLoading] = useState(false);

  const handlePolish = async () => {
    if (!text.trim()) return;
    setLoading(true);
    setResult("");
    try {
      const res = await aiApi.polish(text);
      setResult(res);
      onResult(res);
    } catch (err) {
      setResult(`错误: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-3">
      <textarea
        value={text}
        onChange={(e) => setText(e.target.value)}
        placeholder="输入要润色的文字..."
        rows={4}
        className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
      />
      <button
        onClick={handlePolish}
        disabled={loading || !text.trim()}
        className={cn(
          "w-full rounded-lg px-4 py-2.5 text-sm font-medium text-primary-foreground transition-all",
          "bg-primary hover:bg-primary/90 active:scale-[0.98] disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100",
        )}
      >
        {loading ? "润色中..." : "润色"}
      </button>
      {result && (
        <div className="rounded-lg border border-border bg-muted/50 p-3 text-sm whitespace-pre-wrap animate-fade-in">
          {result}
        </div>
      )}
    </div>
  );
};

const DialoguePanel = ({
  onResult,
}: { onResult: (text: string) => void }) => {
  const [characters, setCharacters] = useState("");
  const [scenario, setScenario] = useState("");
  const [result, setResult] = useState("");
  const [loading, setLoading] = useState(false);

  const handleGenerate = async () => {
    if (!characters.trim() || !scenario.trim()) return;
    setLoading(true);
    setResult("");
    try {
      const res = await aiApi.generateDialogue(characters, scenario);
      setResult(res);
      onResult(res);
    } catch (err) {
      setResult(`错误: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-3">
      <textarea
        value={characters}
        onChange={(e) => setCharacters(e.target.value)}
        placeholder="描述参与对话的角色（名字、性格、关系）..."
        rows={3}
        className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
      />
      <textarea
        value={scenario}
        onChange={(e) => setScenario(e.target.value)}
        placeholder="描述对话发生的场景..."
        rows={2}
        className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
      />
      <button
        onClick={handleGenerate}
        disabled={loading || !characters.trim() || !scenario.trim()}
        className={cn(
          "w-full rounded-lg px-4 py-2.5 text-sm font-medium text-primary-foreground transition-all",
          "bg-primary hover:bg-primary/90 active:scale-[0.98] disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100",
        )}
      >
        {loading ? "生成中..." : "生成对话"}
      </button>
      {result && (
        <div className="rounded-lg border border-border bg-muted/50 p-3 text-sm whitespace-pre-wrap animate-fade-in">
          {result}
        </div>
      )}
    </div>
  );
};

const ChatPanel = ({
  projectId,
  onResult,
}: {
  projectId: string;
  onResult: (text: string) => void;
}) => {
  const [messages, setMessages] = useState<
    Array<{ role: string; content: string }>
  >([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSend = async () => {
    if (!input.trim()) return;
    const userMsg = input.trim();
    setInput("");
    setMessages((prev) => [...prev, { role: "user", content: userMsg }]);
    setLoading(true);
    try {
      const res = await aiApi.chat(projectId, "general", "", userMsg);
      setMessages((prev) => [...prev, { role: "assistant", content: res }]);
      onResult(res);
    } catch (err) {
      setMessages((prev) => [
        ...prev,
        { role: "assistant", content: `错误: ${err}` },
      ]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex-1 overflow-y-auto space-y-3 py-1">
        {messages.length === 0 && (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <div className="mb-3 flex h-10 w-10 items-center justify-center rounded-xl bg-primary/10">
              <MessagesSquare className="h-5 w-5 text-primary/70" />
            </div>
            <p className="text-sm text-muted-foreground">
              向 AI 助手咨询写作相关问题
            </p>
          </div>
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
        {loading && (
          <div className="mr-2 flex items-center gap-2 rounded-xl bg-muted/60 px-3.5 py-2.5 text-sm text-muted-foreground">
            <div className="h-3 w-3 animate-spin rounded-full border-2 border-primary/50 border-t-transparent" />
            思考中...
          </div>
        )}
      </div>
      <div className="flex gap-2 pt-3 border-t border-border">
        <input
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              handleSend();
            }
          }}
          placeholder="输入问题..."
          className="flex-1 rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
        />
        <button
          onClick={handleSend}
          disabled={loading || !input.trim()}
          className={cn(
            "flex items-center justify-center rounded-lg px-3 py-2.5 text-primary-foreground transition-all",
            "bg-primary hover:bg-primary/90 active:scale-[0.97] disabled:opacity-40 disabled:cursor-not-allowed",
          )}
        >
          <Send className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
};

const AiPanel = ({ open, onClose, selectedText, onInsert }: AiPanelProps) => {
  const [activeMode, setActiveMode] = useState<AiMode>("continue");
  const [generatedText, setGeneratedText] = useState("");
  const [copied, setCopied] = useState(false);

  if (!open) return null;

  const handleResult = (text: string) => {
    setGeneratedText(text);
  };

  const handleCopy = async () => {
    await navigator.clipboard.writeText(generatedText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="flex h-full w-[380px] shrink-0 flex-col border-l border-border bg-background animate-fade-in">
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2.5">
          <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-primary/15">
            <Sparkles className="h-3.5 w-3.5 text-primary" />
          </div>
          <span className="text-sm font-semibold text-foreground">AI 助手</span>
        </div>
        <button
          onClick={onClose}
          className="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
        >
          <X className="h-4 w-4" />
        </button>
      </div>

      <div className="flex gap-1 border-b border-border px-3 py-2">
        {modes.map((mode) => (
          <button
            key={mode.key}
            onClick={() => setActiveMode(mode.key)}
            className={cn(
              "flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-xs font-medium transition-all",
              activeMode === mode.key
                ? "bg-primary/15 text-primary"
                : "text-muted-foreground hover:bg-secondary hover:text-foreground active:scale-[0.97]",
            )}
          >
            <mode.icon className="h-3 w-3" />
            {mode.label}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {activeMode === "continue" && (
          <ContinuePanel onResult={handleResult} />
        )}
        {activeMode === "rewrite" && (
          <RewritePanel selectedText={selectedText} onResult={handleResult} />
        )}
        {activeMode === "polish" && (
          <PolishPanel selectedText={selectedText} onResult={handleResult} />
        )}
        {activeMode === "dialogue" && (
          <DialoguePanel onResult={handleResult} />
        )}
        {activeMode === "chat" && (
          <ChatPanel projectId="" onResult={handleResult} />
        )}
      </div>

      {generatedText && (
        <div className="flex gap-2 border-t border-border px-4 py-3">
          <button
            onClick={() => onInsert(generatedText)}
            className={cn(
              "flex flex-1 items-center justify-center gap-2 rounded-lg px-4 py-2.5 text-sm font-medium text-primary-foreground transition-all",
              "bg-primary hover:bg-primary/90 active:scale-[0.98]",
            )}
          >
            <ArrowDownToLine className="h-3.5 w-3.5" />
            插入到编辑器
          </button>
          <button
            onClick={handleCopy}
            className="flex items-center justify-center rounded-lg border border-border px-3 py-2.5 text-muted-foreground transition-all hover:bg-secondary hover:text-foreground active:scale-[0.97]"
          >
            {copied ? (
              <Check className="h-4 w-4 text-primary" />
            ) : (
              <Copy className="h-4 w-4" />
            )}
          </button>
        </div>
      )}
    </div>
  );
};

export default AiPanel;
