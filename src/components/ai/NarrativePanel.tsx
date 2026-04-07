import { useState, useEffect, useRef } from "react";
import { motion } from "motion/react";
import {
  Plus,
  Trash2,
  Play,
  UserRound,
  BookOpen,
  Pencil,
  MessageSquare,
  ChevronRight,
  Sparkles,
  ArrowDownToLine,
  Square,
} from "lucide-react";
import { narrativeApi, characterApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import { springs } from "@/lib/motion";
import { useStreamingSession } from "@/hooks/useStreamingSession";
import type { NarrativeSession, NarrativeBeat, Character, NarrativeStreamChunk } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from "@/components/ui/empty";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { useAiEditor } from "@/contexts/AiEditorContext";

const BeatCard = ({
  beat,
  onDelete,
}: {
  beat: NarrativeBeat;
  onDelete: () => void;
}) => {
  const isNarration = beat.beat_type === "narration";
  const isAuthor = beat.beat_type === "author_intervention";
  const isScene = beat.beat_type === "scene_change";

  return (
    <motion.div
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={springs.snappy}
      className="group/beat relative"
    >
      <div
        className={cn(
          "rounded-xl px-3.5 py-2.5 text-sm leading-relaxed",
          isNarration && "bg-muted/50 text-foreground",
          isAuthor && "border border-primary/25 bg-primary/5 text-foreground",
          isScene && "border border-amber-500/25 bg-amber-500/5 text-foreground",
          !isNarration && !isAuthor && !isScene && "bg-muted/50 text-foreground mr-4",
        )}
      >
        <div className="mb-1 flex items-center gap-1.5">
          {isNarration && (
            <BookOpen className="size-3 text-muted-foreground/50" />
          )}
          {isAuthor && (
            <Pencil className="size-3 text-primary/60" />
          )}
          {isScene && (
            <Sparkles className="size-3 text-amber-500/60" />
          )}
          {!isNarration && !isAuthor && !isScene && (
            <UserRound className="size-3 text-muted-foreground/50" />
          )}
          <span className="text-[11px] font-medium text-muted-foreground/70">
            {isNarration ? "叙事者" : beat.character_name}
          </span>
        </div>
        <p className="whitespace-pre-wrap">{beat.content}</p>
      </div>
      <button
        onClick={onDelete}
        className="absolute -top-1 -right-1 flex size-5 items-center justify-center rounded-full bg-destructive/80 text-destructive-foreground opacity-0 transition-opacity group-hover/beat:opacity-100 hover:bg-destructive"
      >
        <Trash2 className="size-2.5" />
      </button>
    </motion.div>
  );
};

const NarrativePanel = () => {
  const { editorState, replaceSelection } = useAiEditor();
  const [sessions, setSessions] = useState<NarrativeSession[]>([]);
  const [activeSession, setActiveSession] = useState<NarrativeSession | null>(null);
  const [beats, setBeats] = useState<NarrativeBeat[]>([]);
  const [characters, setCharacters] = useState<Character[]>([]);
  const [loading, setLoading] = useState(false);
  const [showCreate, setShowCreate] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newScene, setNewScene] = useState("");

  const [authorInput, setAuthorInput] = useState("");
  const [selectedCharId, setSelectedCharId] = useState("");

  const advance = useStreamingSession();
  const invoke = useStreamingSession();

  const scrollRef = useRef<HTMLDivElement>(null);

  const loadSessions = async () => {
    if (!editorState.projectId) return;
    try {
      const data = await narrativeApi.listSessions(editorState.projectId);
      setSessions(data);
    } catch (err) {
      console.error("Failed to load sessions:", err);
    }
  };

  const loadCharacters = async () => {
    if (!editorState.projectId) return;
    try {
      const data = await characterApi.list(editorState.projectId);
      setCharacters(data);
    } catch (err) {
      console.error("Failed to load characters:", err);
    }
  };

  useEffect(() => {
    loadSessions();
    loadCharacters();
  }, [editorState.projectId]);

  const loadBeats = async (sessionId: string) => {
    setLoading(true);
    try {
      const data = await narrativeApi.listBeats(sessionId);
      setBeats(data);
    } catch (err) {
      console.error("Failed to load beats:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (activeSession) {
      loadBeats(activeSession.id);
    } else {
      setBeats([]);
    }
  }, [activeSession]);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [beats, advance.streamingText, invoke.streamingText]);

  const handleCreateSession = async () => {
    if (!editorState.projectId || !newScene.trim()) return;
    try {
      const session = await narrativeApi.createSession(
        editorState.projectId,
        newTitle.trim() || "未命名推演",
        newScene.trim(),
      );
      setSessions((prev) => [session, ...prev]);
      setActiveSession(session);
      setNewTitle("");
      setNewScene("");
      setShowCreate(false);
    } catch (err) {
      console.error("Failed to create session:", err);
    }
  };

  const handleDeleteSession = async (id: string) => {
    try {
      await narrativeApi.deleteSession(id);
      setSessions((prev) => prev.filter((s) => s.id !== id));
      if (activeSession?.id === id) {
        setActiveSession(null);
      }
    } catch (err) {
      console.error("Failed to delete session:", err);
    }
  };

  const handleAddAuthorBeat = async () => {
    if (!activeSession || !authorInput.trim()) return;
    try {
      const beat = await narrativeApi.addBeat(
        activeSession.id,
        "author_intervention",
        null,
        "作者",
        authorInput.trim(),
      );
      setBeats((prev) => [...prev, beat]);
      setAuthorInput("");
    } catch (err) {
      console.error("Failed to add beat:", err);
    }
  };

  const handleAdvance = async (instruction?: string) => {
    if (!activeSession || advance.isStreaming) return;
    advance.start();

    try {
      await narrativeApi.advanceNarration(
        activeSession.id,
        (chunk: NarrativeStreamChunk) => {
          if (chunk.text) {
            advance.appendText(chunk.text);
          }
          if (chunk.done) {
            advance.finish();
            loadBeats(activeSession.id);
          }
        },
        instruction || null,
      );
    } catch (err) {
      console.error("Failed to advance:", err);
      advance.fail("");
    }
  };

  const handleInvokeCharacter = async () => {
    if (!activeSession || !selectedCharId || invoke.isStreaming) return;
    invoke.start();

    try {
      await narrativeApi.invokeCharacter(
        activeSession.id,
        selectedCharId,
        (chunk: NarrativeStreamChunk) => {
          if (chunk.text) {
            invoke.appendText(chunk.text);
          }
          if (chunk.done) {
            invoke.finish();
            loadBeats(activeSession.id);
          }
        },
        authorInput || null,
      );
    } catch (err) {
      console.error("Failed to invoke character:", err);
      invoke.fail("");
    }
  };

  const handleDeleteBeat = async (id: string) => {
    try {
      await narrativeApi.deleteBeat(id);
      setBeats((prev) => prev.filter((b) => b.id !== id));
    } catch (err) {
      console.error("Failed to delete beat:", err);
    }
  };

  const handleInsertToEditor = () => {
    const fullText = beats
      .filter((b) => b.beat_type !== "author_intervention")
      .map((b) => b.content)
      .join("\n\n");
    if (fullText) {
      replaceSelection(fullText);
    }
  };

  const isStreaming = advance.isStreaming || invoke.isStreaming;
  const streamingText = advance.streamingText || invoke.streamingText;
  const activeStream = advance.isStreaming ? advance : invoke.isStreaming ? invoke : null;
  const streamLabel = advance.isStreaming
    ? "叙事者"
    : invoke.isStreaming
      ? (characters.find((c) => c.id === selectedCharId)?.name || "")
      : "";

  if (!activeSession) {
    return (
      <div className="flex h-full flex-col">
        <div className="flex items-center justify-between px-5 h-11 shrink-0">
          <span className="text-xs font-medium text-foreground">剧情推演</span>
          <Button onClick={() => setShowCreate(true)} size="sm" data-icon="inline-start">
            <Plus />
            新建
          </Button>
        </div>
        <div className="flex-1 overflow-y-auto px-6 py-5">
          {sessions.length === 0 ? (
            <Empty className="py-12 border-transparent">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <BookOpen />
                </EmptyMedia>
                <EmptyTitle>还没有推演会话</EmptyTitle>
                <EmptyDescription>创建一个推演场景开始剧情模拟</EmptyDescription>
                <EmptyContent>
                  <Button onClick={() => setShowCreate(true)} data-icon="inline-start">
                    <Plus />
                    新建推演
                  </Button>
                </EmptyContent>
              </EmptyHeader>
            </Empty>
          ) : (
            <div className="flex flex-col gap-2">
              {sessions.map((session) => (
                <div
                  key={session.id}
                  className="group/session flex items-center gap-3 rounded-lg border border-border px-3 py-2.5 cursor-pointer transition-colors hover:bg-muted/50"
                  onClick={() => setActiveSession(session)}
                >
                  <BookOpen className="size-4 text-muted-foreground/60 shrink-0" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-foreground truncate">
                      {session.title}
                    </p>
                    <p className="text-xs text-muted-foreground truncate">
                      {session.scene}
                    </p>
                  </div>
                  <div className="flex items-center gap-1">
                    <span className="text-[10px] text-muted-foreground/60 mr-1">
                      {session.status === "active" ? "进行中" : session.status === "paused" ? "已暂停" : "已结束"}
                    </span>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDeleteSession(session.id);
                      }}
                      className="flex size-5 items-center justify-center rounded-md text-muted-foreground/40 opacity-0 transition-all group-hover/session:opacity-100 hover:bg-destructive/10 hover:text-destructive"
                    >
                      <Trash2 className="size-3" />
                    </button>
                    <ChevronRight className="size-3.5 text-muted-foreground/40" />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <Dialog open={showCreate} onOpenChange={(isOpen) => { if (!isOpen) setShowCreate(false); }}>
          <DialogContent className="sm:max-w-sm">
            <DialogHeader>
              <DialogTitle>新建推演会话</DialogTitle>
              <DialogDescription>设定一个场景，AI 叙事者将在此场景中推动剧情</DialogDescription>
            </DialogHeader>
            <div className="flex flex-col gap-4">
              <div className="flex flex-col gap-1.5">
                <Label>标题</Label>
                <Input
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  placeholder="推演标题（可选）"
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <Label>初始场景</Label>
                <Textarea
                  value={newScene}
                  onChange={(e) => setNewScene(e.target.value)}
                  placeholder="描述初始场景，如「深夜的酒馆，外面下着大雨」"
                  rows={3}
                  autoFocus
                />
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setShowCreate(false)}>
                取消
              </Button>
              <Button onClick={handleCreateSession} disabled={!newScene.trim()}>
                创建
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-2 px-4 h-11 shrink-0 border-b border-border">
        <button
          onClick={() => setActiveSession(null)}
          className="flex size-6 items-center justify-center rounded-md text-muted-foreground/60 transition-colors hover:bg-secondary hover:text-muted-foreground"
        >
          <ChevronRight className="size-3.5 rotate-180" />
        </button>
        <span className="flex-1 text-xs font-medium text-foreground truncate">
          {activeSession.title}
        </span>
        {isStreaming && (
          <span className="flex items-center gap-1.5 text-[11px] text-primary/60">
            <Spinner className="size-2.5" />
            {advance.isStreaming ? "叙事中" : "角色响应中"}
          </span>
        )}
      </div>

      <div ref={scrollRef} className="flex-1 overflow-y-auto px-4 py-4">
        {loading ? (
          <div className="flex h-32 items-center justify-center">
            <Spinner className="size-5 text-primary" />
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {beats.length === 0 && !isStreaming && (
              <p className="text-xs text-muted-foreground/60 text-center py-8">
                点击「叙事推进」开始剧情
              </p>
            )}
            {beats.map((beat) => (
              <BeatCard
                key={beat.id}
                beat={beat}
                onDelete={() => handleDeleteBeat(beat.id)}
              />
            ))}
            {activeStream && streamingText && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ duration: 0.15 }}
                className={cn(
                  "rounded-xl px-3.5 py-2.5 text-sm leading-relaxed",
                  advance.isStreaming ? "bg-muted/50 text-foreground" : "bg-muted/50 text-foreground mr-4",
                )}
              >
                <div className="mb-1 flex items-center gap-1.5">
                  {advance.isStreaming ? (
                    <BookOpen className="size-3 text-muted-foreground/50" />
                  ) : (
                    <UserRound className="size-3 text-muted-foreground/50" />
                  )}
                  <span className="text-[11px] font-medium text-muted-foreground/70">
                    {streamLabel}
                  </span>
                </div>
                {streamingText}
                <span className="inline-block size-1.5 ml-0.5 rounded-full bg-primary/50 animate-pulse" />
              </motion.div>
            )}
            {isStreaming && !streamingText && (
              <div className="flex items-center gap-2 rounded-xl bg-muted/50 px-3.5 py-2.5 text-sm text-muted-foreground">
                <Spinner className="size-3 text-primary/50" />
                {advance.isStreaming ? "叙事者思考中..." : "角色思考中..."}
              </div>
            )}
          </div>
        )}
      </div>

      <div className="flex flex-col gap-2 border-t border-border px-4 py-3">
        {beats.length > 0 && (
          <Button
            variant="ghost"
            size="xs"
            onClick={handleInsertToEditor}
            className="self-start text-muted-foreground/70"
            data-icon="inline-start"
          >
            <ArrowDownToLine className="size-3" />
            写入正文
          </Button>
        )}

        <div className="flex gap-1.5">
          <Button
            onClick={() => handleAdvance()}
            disabled={isStreaming}
            size="sm"
            className="flex-1"
            data-icon="inline-start"
          >
            {advance.isStreaming ? (
              <><Square className="size-3" />停止</>
            ) : (
              <><Play className="size-3" />叙事推进</>
            )}
          </Button>
        </div>

        {characters.length > 0 && (
          <div className="flex gap-1.5">
            <select
              value={selectedCharId}
              onChange={(e) => setSelectedCharId(e.target.value)}
              className="h-8 rounded-md border border-border bg-background px-2 text-xs text-foreground flex-1 min-w-0"
            >
              <option value="">选择角色...</option>
              {characters.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                </option>
              ))}
            </select>
            <Button
              onClick={handleInvokeCharacter}
              disabled={isStreaming || !selectedCharId}
              size="sm"
              className="flex-1"
              data-icon="inline-start"
            >
              <MessageSquare className="size-3" />
              角色响应
            </Button>
          </div>
        )}

        <Input
          value={authorInput}
          onChange={(e) => setAuthorInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              handleAddAuthorBeat();
            }
          }}
          placeholder="输入作者指令（回车发送）..."
          disabled={isStreaming}
        />
      </div>
    </div>
  );
};

export default NarrativePanel;
