import { useState, useEffect } from "react";
import { motion } from "motion/react";
import {
  Sparkles,
  PenLine,
  MessageSquare,
  MessagesSquare,
  Send,
  Copy,
  Check,
  ArrowDownToLine,
  Square,
  X,
  BookOpen,
} from "lucide-react";

import { aiApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Spinner } from "@/components/ui/spinner";
import { Empty, EmptyHeader, EmptyMedia, EmptyDescription } from "@/components/ui/empty";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { useAiEditor } from "@/contexts/AiEditorContext";
import { springs } from "@/lib/motion";
import type { EditorMode } from "@/contexts/AiEditorContext";
import NarrativePanel from "./NarrativePanel";

interface AiPanelProps {
  open: boolean;
  onClose: () => void;
}

const modes: { key: EditorMode | "narrative"; label: string; icon: typeof Sparkles }[] = [
  { key: "narrative", label: "推演", icon: BookOpen },
  { key: "polish", label: "润色", icon: Sparkles },
  { key: "rewrite", label: "改写", icon: PenLine },
  { key: "dialogue", label: "对话", icon: MessageSquare },
  { key: "chat", label: "聊天", icon: MessagesSquare },
];

const PolishPanel = () => {
  const { editorState, startStreaming, isStreaming, streamingText, generatedText, stopStreaming, replaceSelection } = useAiEditor();
  const [copied, setCopied] = useState(false);

  const text = editorState.selectedText || streamingText || generatedText || "";
  const finalText = generatedText || streamingText;
  const hasContent = text.trim().length > 0;

  const handlePolish = () => {
    if (!text.trim()) return;
    startStreaming({ text });
  };

  const handleCopy = async () => {
    if (!finalText) return;
    await navigator.clipboard.writeText(finalText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleInsert = () => {
    if (!finalText) return;
    replaceSelection(finalText);
  };

  return (
    <div className="flex flex-col gap-3">
      {editorState.hasSelection ? (
        <div className="rounded-lg border border-primary/20 bg-primary/5 p-3 text-xs leading-relaxed">
          <div className="mb-1.5 text-[11px] font-medium text-primary/70 uppercase tracking-wider">选中文本</div>
          <div className="text-muted-foreground max-h-32 overflow-y-auto">
            {editorState.selectedText.slice(0, 400)}
            {editorState.selectedText.length > 400 && "..."}
          </div>
        </div>
      ) : (
        <div className="rounded-lg border border-border bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
          在编辑器中选中文字后使用润色
        </div>
      )}

      {finalText && !isStreaming && (
        <div className="rounded-lg border border-border bg-muted/30 p-3 text-sm leading-relaxed max-h-48 overflow-y-auto">
          {finalText}
        </div>
      )}

      {isStreaming && streamingText && (
        <div className="rounded-lg border border-primary/20 bg-primary/5 p-3 text-sm leading-relaxed max-h-48 overflow-y-auto">
          {streamingText}
          <span className="inline-block size-1.5 ml-0.5 rounded-full bg-primary/50 animate-pulse" />
        </div>
      )}

      <div className="flex gap-2">
        {!finalText && !isStreaming && (
          <Button
            onClick={handlePolish}
            disabled={isStreaming || !hasContent}
            className="flex-1"
            data-icon="inline-start"
          >
            <Sparkles />
            润色
          </Button>
        )}
        {isStreaming && (
          <Button variant="outline" onClick={stopStreaming} className="flex-1" data-icon="inline-start">
            <Square className="size-3" />
            停止
          </Button>
        )}
        {finalText && !isStreaming && (
          <>
            <Button onClick={handleInsert} className="flex-1" data-icon="inline-start">
              <ArrowDownToLine />
              插入
            </Button>
            <Button variant="outline" size="icon" onClick={handleCopy}>
              {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
            </Button>
          </>
        )}
      </div>
    </div>
  );
};

const RewritePanel = () => {
  const { editorState, startStreaming, isStreaming, streamingText, generatedText, stopStreaming, replaceSelection } = useAiEditor();
  const [instruction, setInstruction] = useState("");
  const [copied, setCopied] = useState(false);

  const text = editorState.selectedText || "";
  const finalText = generatedText || streamingText;

  const handleRewrite = () => {
    if (!text.trim()) return;
    startStreaming({
      text,
      instruction: instruction || undefined,
    });
  };

  const handleCopy = async () => {
    if (!finalText) return;
    await navigator.clipboard.writeText(finalText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleInsert = () => {
    if (!finalText) return;
    replaceSelection(finalText);
  };

  return (
    <div className="flex flex-col gap-3">
      {text ? (
        <div className="rounded-lg border border-primary/20 bg-primary/5 p-3 text-xs leading-relaxed">
          <div className="mb-1.5 text-[11px] font-medium text-primary/70 uppercase tracking-wider">选中文本</div>
          <div className="text-muted-foreground max-h-32 overflow-y-auto">
            {text.slice(0, 400)}
            {text.length > 400 && "..."}
          </div>
        </div>
      ) : (
        <div className="rounded-lg border border-border bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
          在编辑器中选中文字后使用改写
        </div>
      )}

      <Input
        value={instruction}
        onChange={(e) => setInstruction(e.target.value)}
        placeholder="改写指令（可选，如：更正式、更生动...）"
      />

      {finalText && !isStreaming && (
        <div className="rounded-lg border border-border bg-muted/30 p-3 text-sm leading-relaxed max-h-48 overflow-y-auto">
          {finalText}
        </div>
      )}

      {isStreaming && streamingText && (
        <div className="rounded-lg border border-primary/20 bg-primary/5 p-3 text-sm leading-relaxed max-h-48 overflow-y-auto">
          {streamingText}
          <span className="inline-block size-1.5 ml-0.5 rounded-full bg-primary/50 animate-pulse" />
        </div>
      )}

      <div className="flex gap-2">
        {!finalText && !isStreaming && (
          <Button
            onClick={handleRewrite}
            disabled={isStreaming || !text.trim()}
            className="flex-1"
            data-icon="inline-start"
          >
            <PenLine />
            改写
          </Button>
        )}
        {isStreaming && (
          <Button variant="outline" onClick={stopStreaming} className="flex-1" data-icon="inline-start">
            <Square className="size-3" />
            停止
          </Button>
        )}
        {finalText && !isStreaming && (
          <>
            <Button onClick={handleInsert} className="flex-1" data-icon="inline-start">
              <ArrowDownToLine />
              替换选中
            </Button>
            <Button variant="outline" size="icon" onClick={handleCopy}>
              {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
            </Button>
          </>
        )}
      </div>
    </div>
  );
};

const DialoguePanel = () => {
  const { startStreaming, isStreaming, streamingText, generatedText, stopStreaming, replaceSelection } = useAiEditor();
  const [characters, setCharacters] = useState("");
  const [scenario, setScenario] = useState("");
  const [copied, setCopied] = useState(false);

  const finalText = generatedText || streamingText;

  const handleGenerate = () => {
    if (!characters.trim() || !scenario.trim()) return;
    const text = `角色信息：\n${characters}\n\n场景描述：\n${scenario}`;
    startStreaming({ text });
  };

  const handleCopy = async () => {
    if (!finalText) return;
    await navigator.clipboard.writeText(finalText);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleInsert = () => {
    if (!finalText) return;
    replaceSelection(finalText);
  };

  return (
    <div className="flex flex-col gap-3">
      <Textarea
        value={characters}
        onChange={(e) => setCharacters(e.target.value)}
        placeholder="角色信息（名字、性格、关系）..."
        rows={3}
      />
      <Textarea
        value={scenario}
        onChange={(e) => setScenario(e.target.value)}
        placeholder="场景描述..."
        rows={2}
      />

      {finalText && !isStreaming && (
        <div className="rounded-lg border border-border bg-muted/30 p-3 text-sm leading-relaxed max-h-48 overflow-y-auto">
          {finalText}
        </div>
      )}

      {isStreaming && streamingText && (
        <div className="rounded-lg border border-primary/20 bg-primary/5 p-3 text-sm leading-relaxed max-h-48 overflow-y-auto">
          {streamingText}
          <span className="inline-block size-1.5 ml-0.5 rounded-full bg-primary/50 animate-pulse" />
        </div>
      )}

      <div className="flex gap-2">
        {!finalText && !isStreaming && (
          <Button
            onClick={handleGenerate}
            disabled={isStreaming || !characters.trim() || !scenario.trim()}
            className="flex-1"
            data-icon="inline-start"
          >
            <MessageSquare />
            生成对话
          </Button>
        )}
        {isStreaming && (
          <Button variant="outline" onClick={stopStreaming} className="flex-1" data-icon="inline-start">
            <Square className="size-3" />
            停止
          </Button>
        )}
        {finalText && !isStreaming && (
          <>
            <Button onClick={handleInsert} className="flex-1" data-icon="inline-start">
              <ArrowDownToLine />
              插入
            </Button>
            <Button variant="outline" size="icon" onClick={handleCopy}>
              {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
            </Button>
          </>
        )}
      </div>
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

    startStreaming({ text: userMsg });
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
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 8 }}
            animate={{ opacity: 1, y: 0 }}
            transition={springs.snappy}
            className={cn(
              "rounded-xl px-3.5 py-2.5 text-sm leading-relaxed",
              msg.role === "user"
                ? "bg-primary/12 text-foreground ml-6"
                : "bg-muted/60 text-foreground mr-2",
            )}
          >
            {msg.content}
          </motion.div>
        ))}
        {isStreaming && streamingText && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ duration: 0.15 }}
            className="mr-2 rounded-xl bg-muted/60 px-3.5 py-2.5 text-sm leading-relaxed"
          >
            {streamingText}
            <span className="inline-block size-1.5 ml-0.5 rounded-full bg-primary/50 animate-pulse" />
          </motion.div>
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

const AiPanel = ({ open: _open, onClose }: AiPanelProps) => {
  const { activeMode, setActiveMode, isStreaming } = useAiEditor();
  const [narrativeMode, setNarrativeMode] = useState(false);

  const handleModeChange = (v: string) => {
    if (v === "narrative") {
      setNarrativeMode(true);
    } else {
      setNarrativeMode(false);
      setActiveMode(v as EditorMode);
    }
  };

  return (
    <motion.div
      initial={{ x: "100%", opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      exit={{ x: "100%", opacity: 0 }}
      transition={springs.smooth}
      className="flex h-full w-[360px] shrink-0 flex-col border-l border-border bg-background overflow-hidden"
    >
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border shrink-0">
        <span className="text-xs font-medium text-foreground">AI 助手</span>
        <div className="flex items-center gap-1.5">
          {isStreaming && (
            <span className="flex items-center gap-1.5 text-[11px] text-primary/60">
              <Spinner className="size-2.5" />
              输出中
            </span>
          )}
          <button
            type="button"
            onClick={onClose}
            className="flex size-6 items-center justify-center rounded-md text-muted-foreground/60 transition-colors hover:bg-secondary hover:text-muted-foreground"
          >
            <X className="size-3.5" />
          </button>
        </div>
      </div>

      <Tabs
        value={narrativeMode ? "narrative" : activeMode}
        onValueChange={handleModeChange}
        className="flex flex-col flex-1 overflow-hidden"
      >
        <div className="border-b border-border px-3 py-1.5 shrink-0">
          <TabsList variant="line">
            {modes.map((mode) => (
              <TabsTrigger key={mode.key} value={mode.key} data-icon="inline-start">
                <mode.icon />
                {mode.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </div>

        <div className="flex-1 overflow-hidden">
          <TabsContent value="narrative" className="h-full overflow-hidden">
            <NarrativePanel />
          </TabsContent>
          <TabsContent value="polish"><PolishPanel /></TabsContent>
          <TabsContent value="rewrite"><RewritePanel /></TabsContent>
          <TabsContent value="dialogue"><DialoguePanel /></TabsContent>
          <TabsContent value="chat"><ChatPanel /></TabsContent>
        </div>
      </Tabs>
    </motion.div>
  );
};

export default AiPanel;
