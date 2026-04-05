import { useState, useEffect, useRef } from "react";
import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/shadcn";
import { Sparkles, Save, FileText } from "lucide-react";
import type { PartialBlock, Block } from "@blocknote/core";
import { outlineApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { OutlineNode, ChapterStatus } from "@/lib/types";
import AiPanel from "@/components/ai/AiPanel";

interface EditorProps {
  chapterId: string;
}

const extractTextFromBlock = (block: Block): string => {
  let text = "";
  if (block.content && Array.isArray(block.content)) {
    for (const inline of block.content) {
      if (typeof inline === "string") {
        text += inline;
      } else if (inline.type === "text") {
        text += inline.text;
      } else if (inline.type === "link") {
        if (Array.isArray(inline.content)) {
          for (const linkInline of inline.content) {
            if (typeof linkInline === "string") {
              text += linkInline;
            } else {
              text += linkInline.text;
            }
          }
        } else if (typeof inline.content === "string") {
          text += inline.content;
        }
      }
    }
  }
  return text;
};

const countWords = (blocks: Block[]): number => {
  let count = 0;
  for (const block of blocks) {
    const text = extractTextFromBlock(block);
    if (text.trim().length > 0) {
      const trimmed = text.trim();
      const chinese = trimmed.match(/[\u4e00-\u9fff]/g);
      const english = trimmed
        .replace(/[\u4e00-\u9fff]/g, " ")
        .trim()
        .split(/\s+/)
        .filter(Boolean);
      count += (chinese?.length || 0) + english.length;
    }
    if (block.children && block.children.length > 0) {
      count += countWords(block.children);
    }
  }
  return count;
};

const statusConfig: Record<ChapterStatus, { label: string; className: string }> = {
  draft: {
    label: "草稿",
    className: "bg-muted/80 text-muted-foreground",
  },
  completed: {
    label: "已完成",
    className: "bg-emerald-500/12 text-emerald-400",
  },
  revising: {
    label: "修订中",
    className: "bg-amber-500/12 text-amber-400",
  },
};

const Editor = ({ chapterId }: EditorProps) => {
  const [node, setNode] = useState<OutlineNode | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [title, setTitle] = useState("");
  const [status, setStatus] = useState<ChapterStatus>("draft");
  const [aiPanelOpen, setAiPanelOpen] = useState(false);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const editor = useCreateBlockNote({
    initialContent: [
      {
        type: "paragraph",
        content: "",
      },
    ] as PartialBlock[],
  });

  const saveChapter = async (blocks: Block[]) => {
    if (!node) return;
    const wordCount = countWords(blocks);
    const contentJson = JSON.stringify(blocks);
    try {
      setSaving(true);
      const updated = await outlineApi.update(
        node.id,
        title,
        contentJson,
        wordCount,
        status,
      );
      setNode(updated);
    } catch (err) {
      console.error("Failed to save chapter:", err);
    } finally {
      setSaving(false);
    }
  };

  const scheduleSave = (blocks: Block[]) => {
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    saveTimerRef.current = setTimeout(() => {
      saveChapter(blocks);
    }, 1500);
  };

  const handleTitleChange = (newTitle: string) => {
    setTitle(newTitle);
    scheduleSave(editor.document);
  };

  const handleEditorChange = () => {
    scheduleSave(editor.document);
  };

  const handleStatusChange = (newStatus: ChapterStatus) => {
    setStatus(newStatus);
    if (node) {
      const wordCount = countWords(editor.document);
      const contentJson = JSON.stringify(editor.document);
      outlineApi.update(node.id, title, contentJson, wordCount, newStatus);
    }
  };

  useEffect(() => {
    const loadNode = async () => {
      setLoading(true);
      try {
        const n = await outlineApi.get(chapterId);
        setNode(n);
        setTitle(n.title);
        setStatus(n.status as ChapterStatus);

        if (n.content_json && n.content_json !== "[]") {
          try {
            const blocks = JSON.parse(n.content_json) as PartialBlock[];
            if (blocks.length > 0) {
              await editor.replaceBlocks(
                editor.document.map((b) => b.id),
                blocks,
              );
            }
          } catch {
            // ignore parse errors
          }
        }
      } catch (err) {
        console.error("Failed to load chapter:", err);
      } finally {
        setLoading(false);
      }
    };
    loadNode();
  }, [chapterId]);

  useEffect(() => {
    return () => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    };
  }, []);

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

  const wordCount = countWords(editor.document);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-border px-6 py-2.5">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          <input
            type="text"
            value={title}
            onChange={(e) => handleTitleChange(e.target.value)}
            className="flex-1 min-w-0 bg-transparent text-lg font-semibold text-foreground outline-none placeholder:text-muted-foreground/40 transition-colors"
            placeholder="章节标题..."
          />
        </div>
        <div className="flex items-center gap-2 shrink-0 ml-4">
          <select
            value={status}
            onChange={(e) => handleStatusChange(e.target.value as ChapterStatus)}
            className="rounded-lg border border-border bg-card px-2.5 py-1 text-xs outline-none transition-colors hover:border-primary/30 cursor-pointer"
            style={{ colorScheme: "dark" }}
          >
            {Object.entries(statusConfig).map(([key, { label }]) => (
              <option key={key} value={key} style={{ colorScheme: "dark" }}>
                {label}
              </option>
            ))}
          </select>
          <span className={cn(
            "rounded-full px-2 py-0.5 text-xs font-medium",
            statusConfig[status].className,
          )}>
            {statusConfig[status].label}
          </span>
          <div className="h-4 w-px bg-border" />
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
            <FileText className="h-3 w-3" />
            <span>{wordCount} 字</span>
          </div>
          {saving && (
            <div className="flex items-center gap-1 text-xs text-primary/70">
              <Save className="h-3 w-3 animate-pulse-subtle" />
              <span>保存中</span>
            </div>
          )}
          <div className="h-4 w-px bg-border" />
          <button
            onClick={() => setAiPanelOpen(!aiPanelOpen)}
            className={cn(
              "flex items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-medium transition-all",
              aiPanelOpen
                ? "bg-primary/15 text-primary"
                : "text-muted-foreground hover:bg-secondary hover:text-foreground active:scale-[0.97]",
            )}
          >
            <Sparkles className="h-3.5 w-3.5" />
            AI
          </button>
        </div>
      </div>
      <div className="flex flex-1 overflow-hidden">
        <div className="flex-1 overflow-y-auto">
          <div className="mx-auto max-w-3xl px-8 py-10">
            <BlockNoteView
              editor={editor}
              onChange={handleEditorChange}
            />
          </div>
        </div>
        <AiPanel
          open={aiPanelOpen}
          onClose={() => setAiPanelOpen(false)}
          selectedText=""
          onInsert={(text: string) => {
            if (editor) {
              const lastBlock = editor.document[editor.document.length - 1];
              editor.insertBlocks(
                [
                  {
                    type: "paragraph" as const,
                    content: text,
                  },
                ],
                lastBlock,
              );
            }
          }}
        />
      </div>
    </div>
  );
};

export default Editor;
