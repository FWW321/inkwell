import { useState, useEffect, useRef, useCallback } from "react";
import { AnimatePresence } from "motion/react";
import { useEditor, EditorContent } from "@tiptap/react";
import { BubbleMenu } from "@tiptap/react/menus";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import Highlight from "@tiptap/extension-highlight";
import TextAlign from "@tiptap/extension-text-align";
import CharacterCount from "@tiptap/extension-character-count";
import Typography from "@tiptap/extension-typography";
import { InlineDiff, inlineDiffPluginKey, setDiffText, getDiffText, setDiffReady } from "@/extensions/inline-diff";
import {
  Sparkles,
  Save,
  FileText,
  Bold,
  Italic,
  Underline as UnderlineIcon,
  Strikethrough,
  Code,
  Heading1,
  Heading2,
  Heading3,
  AlignLeft,
  AlignCenter,
  AlignRight,
  List,
  ListOrdered,
  Quote,
  Minus,
  Undo,
  Redo,
  Wand2,
  Paintbrush,
  PenLine,
  Highlighter,
  Check,
  X,
} from "lucide-react";
import { outlineApi } from "@/lib/api";
import type { OutlineNode, ChapterStatus } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Spinner } from "@/components/ui/spinner";
import { Select, SelectTrigger, SelectContent, SelectGroup, SelectItem, SelectValue } from "@/components/ui/select";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";
import { useEditorSelection } from "@/hooks/useEditorSelection";
import { useAiEditor } from "@/contexts/AiEditorContext";
import AiPanel from "@/components/ai/AiPanel";
import AiFloatingPanel from "@/components/editor/AiFloatingPanel";
import { cn } from "@/lib/utils";

interface EditorProps {
  chapterId: string;
}

const statusConfig: Record<ChapterStatus, { label: string; dot: string }> = {
  draft: { label: "草稿", dot: "bg-muted-foreground/50" },
  in_progress: { label: "写作中", dot: "bg-blue-500" },
  complete: { label: "已完成", dot: "bg-green-500" },
  revised: { label: "已修订", dot: "bg-amber-500" },
};

const countWords = (text: string): number => {
  const trimmed = text.trim();
  if (!trimmed) return 0;
  const chinese = trimmed.match(/[\u4e00-\u9fff]/g);
  const english = trimmed
    .replace(/[\u4e00-\u9fff]/g, " ")
    .trim()
    .split(/\s+/)
    .filter(Boolean);
  return (chinese?.length || 0) + english.length;
};

const Editor = ({ chapterId }: EditorProps) => {
  const [node, setNode] = useState<OutlineNode | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [title, setTitle] = useState("");
  const [status, setStatus] = useState<ChapterStatus>("draft");
  const [aiPanelOpen, setAiPanelOpen] = useState(false);
  const [aiDiffActive, setAiDiffActive] = useState(false);
  const [originalText, setOriginalText] = useState("");
  const [anchorFrom, setAnchorFrom] = useState(0);
  const [anchorTo, setAnchorTo] = useState(0);
  const [actionBarPos, setActionBarPos] = useState<{ top: number; left: number } | null>(null);
  const [cursorOnDiff, setCursorOnDiff] = useState(false);
  const diffSaveRef = useRef(false);
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const editor = useEditor({
    extensions: [
      StarterKit.configure({
        heading: { levels: [1, 2, 3] },
      }),
      Placeholder.configure({
        placeholder: "开始写作...",
      }),
      Highlight.configure({ multicolor: false }),
      TextAlign.configure({ types: ["heading", "paragraph"] }),
      CharacterCount,
      Typography,
      InlineDiff,
    ],
    editorProps: {
      attributes: {
        class: "inkwell-editor outline-none min-h-full",
      },
      handleKeyDown: (_view, _event) => {
        return false;
      },
    },
  });

  const {
    setProjectInfo,
    setActiveMode,
    startStreaming,
    setEditorRef,
    streamingText,
    generatedText,
    setGeneratedText,
    isStreaming,
    stopStreaming,
    activeMode,
  } = useAiEditor();

  const { state: selState, setProjectInfo: setSelProjectInfo } =
    useEditorSelection(editor);

  useEffect(() => {
    if (editor) setEditorRef(editor);
  }, [editor, setEditorRef]);

  const clearDiffState = useCallback(() => {
    if (aiDiffActive && node) {
      outlineApi.clearDiff(node.id).catch(() => {});
    }
    setAiDiffActive(false);
    setOriginalText("");
    setGeneratedText("");
    setAnchorFrom(0);
    setAnchorTo(0);
    setActionBarPos(null);
    setCursorOnDiff(false);
    diffSaveRef.current = false;
    if (editor) {
      (editor.commands as any).clearInlineDiff();
    }
  }, [editor, aiDiffActive, node, setGeneratedText]);

  const handleAccept = useCallback(() => {
    if (!editor || !aiDiffActive) return;
    const text = getDiffText();
    if (!text) return;
    editor.chain().focus()
      .setTextSelection({ from: anchorFrom, to: anchorTo })
      .deleteSelection()
      .insertContent(text)
      .run();
    clearDiffState();
  }, [editor, aiDiffActive, anchorFrom, anchorTo, clearDiffState]);

  const handleReject = useCallback(() => {
    if (isStreaming) stopStreaming();
    clearDiffState();
  }, [isStreaming, stopStreaming, clearDiffState]);

  const saveChapter = useCallback(
    (contentJson: string, wordCount: number) => {
      if (!node) return;
      setSaving(true);
      outlineApi
        .update(node.id, title, contentJson, wordCount, status)
        .then((updated) => setNode(updated))
        .catch((err) => console.error("Failed to save chapter:", err))
        .finally(() => setSaving(false));
    },
    [node, title, status],
  );

  const scheduleSave = useCallback(
    (contentJson: string, wordCount: number) => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
      saveTimerRef.current = setTimeout(() => {
        saveChapter(contentJson, wordCount);
      }, 1500);
    },
    [saveChapter],
  );

  const handleEditorUpdate = useCallback(() => {
    if (!editor) return;

    if (aiDiffActive) {
      const ps = inlineDiffPluginKey.getState(editor.state);
      if (!ps || ps.from === ps.to) {
        clearDiffState();
      } else if (ps.from !== anchorFrom || ps.to !== anchorTo) {
        setAnchorFrom(ps.from);
        setAnchorTo(ps.to);
      }
    }

    const json = JSON.stringify(editor.getJSON());
    const text = editor.state.doc.textContent;
    const wordCount = countWords(text);
    scheduleSave(json, wordCount);
  }, [editor, aiDiffActive, anchorFrom, anchorTo, clearDiffState, scheduleSave]);

  const handleTitleChange = (newTitle: string) => {
    setTitle(newTitle);
    if (editor) {
      const json = JSON.stringify(editor.getJSON());
      const text = editor.state.doc.textContent;
      const wordCount = countWords(text);
      scheduleSave(json, wordCount);
    }
  };

  const handleStatusChange = (newStatus: string | null) => {
    if (!newStatus) return;
    const s = newStatus as ChapterStatus;
    setStatus(s);
    if (editor && node) {
      const json = JSON.stringify(editor.getJSON());
      const text = editor.state.doc.textContent;
      const wordCount = countWords(text);
      outlineApi.update(node.id, title, json, wordCount, s);
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

        setProjectInfo(n.project_id, n.id);
        setSelProjectInfo(n.project_id, n.id);

        if (n.content_json && Object.keys(n.content_json).length > 0) {
          try {
            editor.commands.setContent(n.content_json);
          } catch {
          }
        }

        if (n.diff_original && n.diff_new && n.diff_mode) {
          const diffOrig = n.diff_original;
          const diffMode = n.diff_mode as "polish" | "rewrite";
          requestAnimationFrame(() => {
            const text = editor.state.doc.textContent;
            const idx = text.indexOf(diffOrig);
            if (idx === -1) {
              outlineApi.clearDiff(n.id).catch(() => {});
              return;
            }
            const from = idx + 1;
            const to = from + diffOrig.length;
            setAnchorFrom(from);
            setAnchorTo(to);
            setOriginalText(diffOrig);
            setAiDiffActive(true);
            setActiveMode(diffMode);
            diffSaveRef.current = true;
            (editor.commands as any).setInlineDiff(from, to);
            setDiffText(n.diff_new ?? "");
            setGeneratedText(n.diff_new ?? "");
            setDiffReady(true);
          });
        }
      } catch (err) {
        console.error("Failed to load chapter:", err);
      } finally {
        setLoading(false);
      }
    };
    loadNode();
  }, [editor, chapterId]);

  useEffect(() => {
    return () => {
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
    };
  }, []);

  const diffHitTest = useCallback((clientX: number, clientY: number) => {
    const orig = editor?.view.dom.querySelector("[data-inline-diff='original']") as HTMLElement | null;
    if (!orig) return false;
    const rect = orig.getBoundingClientRect();
    if (clientX >= rect.left && clientX <= rect.right && clientY >= rect.top - 48 && clientY <= rect.bottom) {
      return true;
    }
    const action = document.querySelector("[data-diff-action]") as HTMLElement | null;
    if (action) {
      const ar = action.getBoundingClientRect();
      if (clientX >= ar.left && clientX <= ar.right && clientY >= ar.top && clientY <= ar.bottom) {
        return true;
      }
    }
    return false;
  }, [editor]);

  useEffect(() => {
    if (!aiDiffActive || isStreaming || !generatedText) return;
    const onMove = (e: PointerEvent) => {
      setCursorOnDiff(diffHitTest(e.clientX, e.clientY));
    };
    document.addEventListener("pointermove", onMove, { passive: true });
    return () => document.removeEventListener("pointermove", onMove);
  }, [aiDiffActive, isStreaming, generatedText, diffHitTest]);

  useEffect(() => {
    if (!editor) return;
    editor.on("update", handleEditorUpdate);
    return () => {
      editor.off("update", handleEditorUpdate);
    };
  }, [editor, handleEditorUpdate]);

  useEffect(() => {
    if (!node || !aiDiffActive || isStreaming || !generatedText || !originalText) return;
    if (diffSaveRef.current) return;
    diffSaveRef.current = true;
    outlineApi.saveDiff(node.id, originalText, generatedText, activeMode).catch(() => {});
  }, [node, aiDiffActive, isStreaming, generatedText, originalText, activeMode]);

  useEffect(() => {
    if (!aiDiffActive) return;
    const text = generatedText || streamingText;
    if (text) setDiffText(text);
  }, [streamingText, generatedText, aiDiffActive]);

  useEffect(() => {
    if (!aiDiffActive || !editor) return;
    if (!isStreaming && generatedText) {
      setDiffReady(true);
    }
  }, [aiDiffActive, editor, isStreaming, generatedText]);

  useEffect(() => {
    if (!aiDiffActive || !editor || isStreaming) {
      setActionBarPos(null);
      return;
    }
    if (!generatedText || !cursorOnDiff) {
      setActionBarPos(null);
      return;
    }

    const update = () => {
      const parent = editor.view.dom.closest(".relative") as HTMLElement | null;
      if (!parent) return;
      const parentRect = parent.getBoundingClientRect();
      const original = editor.view.dom.querySelector("[data-inline-diff='original']") as HTMLElement | null;
      if (!original) return;
      const rect = original.getBoundingClientRect();
      setActionBarPos({
        top: rect.top - parentRect.top - 36,
        left: Math.max(8, rect.right - parentRect.left - 160),
      });
    };

    update();
    const timer = setInterval(update, 500);
    const scrollParent = editor.view.dom.closest(".overflow-y-auto") as HTMLElement | null;
    const onScroll = () => requestAnimationFrame(update);
    scrollParent?.addEventListener("scroll", onScroll, { passive: true });

    return () => {
      clearInterval(timer);
      scrollParent?.removeEventListener("scroll", onScroll);
    };
  }, [aiDiffActive, editor, isStreaming, generatedText, cursorOnDiff]);

  const handleFloatingAction = useCallback(
    (mode: "polish" | "rewrite") => {
      const selectedText = selState.selectedText;
      if (!selectedText || !editor) return;

      const { from, to } = editor.state.selection;
      setAnchorFrom(from);
      setAnchorTo(to);
      setOriginalText(selectedText);
      setAiDiffActive(true);
      setActiveMode(mode);
      editor.commands.setTextSelection({ from: to, to: to });
      (editor.commands as any).setInlineDiff(from, to);

      if (mode === "polish") {
        startStreaming({ text: selectedText });
      } else {
        startStreaming({ text: selectedText, instruction: "" });
      }
    },
    [selState.selectedText, editor, setActiveMode, startStreaming],
  );

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

  const wordCount = editor ? countWords(editor.state.doc.textContent) : 0;

  const fmtBtn = (
    Icon: typeof Bold,
    label: string,
    action: () => void,
    active?: boolean,
  ) => (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant={active ? "secondary" : "ghost"}
            size="icon-xs"
            onClick={action}
            data-icon
          >
            <Icon />
          </Button>
        }
      />
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  );

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center gap-3 border-b border-border px-5 h-11 shrink-0">
        <input
          type="text"
          value={title}
          onChange={(e) => handleTitleChange(e.target.value)}
          className="flex-1 min-w-0 bg-transparent text-sm font-medium text-foreground outline-none placeholder:text-muted-foreground/40"
          placeholder="章节标题..."
        />
        <Button
          variant={aiPanelOpen ? "secondary" : "ghost"}
          size="sm"
          onClick={() => setAiPanelOpen(!aiPanelOpen)}
          data-icon="inline-start"
          className="shrink-0"
        >
          <Sparkles />
          AI
        </Button>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto">
            <div className="mx-auto max-w-3xl px-8 py-10 relative">
              {editor && (
                <BubbleMenu
                  editor={editor}
                  options={{ placement: "top", offset: 8 }}
                  shouldShow={({ editor: e, state: s }) => {
                    const { from, to } = s.selection;
                    if (from === to) return false;
                    if (e.view.composing) return false;
                    return true;
                  }}
                >
                  <div className="flex items-center gap-0.5 rounded-xl border border-border bg-popover/95 backdrop-blur-sm px-1 py-1 shadow-lg">
                    {fmtBtn(Undo, "撤销", () => editor.chain().focus().undo().run())}
                    {fmtBtn(Redo, "重做", () => editor.chain().focus().redo().run())}
                    <Separator orientation="vertical" className="mx-0.5 !h-4" />
                    <Tooltip>
                      <TooltipTrigger
                        render={
                          <Select>
                            <SelectTrigger size="sm" className="w-auto px-1">
                              <SelectValue placeholder="">
                                {editor?.isActive("heading", { level: 1 }) ? <Heading1 className="size-3.5" /> : editor?.isActive("heading", { level: 2 }) ? <Heading2 className="size-3.5" /> : editor?.isActive("heading", { level: 3 }) ? <Heading3 className="size-3.5" /> : <span className="text-xs font-medium">正文</span>}
                              </SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                              <SelectGroup>
                                <SelectItem value="paragraph" onClick={() => editor?.chain().focus().setParagraph().run()}>正文</SelectItem>
                                <SelectItem value="heading1" onClick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}>标题 1</SelectItem>
                                <SelectItem value="heading2" onClick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}>标题 2</SelectItem>
                                <SelectItem value="heading3" onClick={() => editor?.chain().focus().toggleHeading({ level: 3 }).run()}>标题 3</SelectItem>
                              </SelectGroup>
                            </SelectContent>
                          </Select>
                        }
                      />
                      <TooltipContent>段落样式</TooltipContent>
                    </Tooltip>
                    <Separator orientation="vertical" className="mx-0.5 !h-4" />
                    {fmtBtn(Bold, "加粗", () => editor.chain().focus().toggleBold().run(), editor?.isActive("bold"))}
                    {fmtBtn(Italic, "斜体", () => editor.chain().focus().toggleItalic().run(), editor?.isActive("italic"))}
                    {fmtBtn(UnderlineIcon, "下划线", () => editor.chain().focus().toggleUnderline().run(), editor?.isActive("underline"))}
                    {fmtBtn(Strikethrough, "删除线", () => editor.chain().focus().toggleStrike().run(), editor?.isActive("strike"))}
                    {fmtBtn(Code, "行内代码", () => editor.chain().focus().toggleCode().run(), editor?.isActive("code"))}
                    {fmtBtn(Highlighter, "高亮", () => editor.chain().focus().toggleHighlight().run(), editor?.isActive("highlight"))}
                    <Separator orientation="vertical" className="mx-0.5 !h-4" />
                    {fmtBtn(AlignLeft, "左对齐", () => editor.chain().focus().setTextAlign("left").run(), editor?.isActive({ textAlign: "left" }))}
                    {fmtBtn(AlignCenter, "居中", () => editor.chain().focus().setTextAlign("center").run(), editor?.isActive({ textAlign: "center" }))}
                    {fmtBtn(AlignRight, "右对齐", () => editor.chain().focus().setTextAlign("right").run(), editor?.isActive({ textAlign: "right" }))}
                    <Separator orientation="vertical" className="mx-0.5 !h-4" />
                    {fmtBtn(List, "无序列表", () => editor.chain().focus().toggleBulletList().run(), editor?.isActive("bulletList"))}
                    {fmtBtn(ListOrdered, "有序列表", () => editor.chain().focus().toggleOrderedList().run(), editor?.isActive("orderedList"))}
                    {fmtBtn(Quote, "引用", () => editor.chain().focus().toggleBlockquote().run(), editor?.isActive("blockquote"))}
                    {fmtBtn(Minus, "分割线", () => editor.chain().focus().setHorizontalRule().run())}
                    {!aiDiffActive && (
                      <>
                        <Separator orientation="vertical" className="mx-0.5 !h-4" />
                        <Tooltip>
                          <TooltipTrigger
                            render={
                              <Button
                                variant="ghost"
                                size="xs"
                                onClick={() => handleFloatingAction("polish")}
                                data-icon="inline-start"
                                className="text-xs text-primary hover:text-primary"
                              >
                                <Paintbrush />
                                润色
                              </Button>
                            }
                          />
                          <TooltipContent>AI 润色选中文字</TooltipContent>
                        </Tooltip>
                        <Tooltip>
                          <TooltipTrigger
                            render={
                              <Button
                                variant="ghost"
                                size="xs"
                                onClick={() => handleFloatingAction("rewrite")}
                                data-icon="inline-start"
                                className="text-xs text-primary hover:text-primary"
                              >
                                <Wand2 />
                                改写
                              </Button>
                            }
                          />
                          <TooltipContent>AI 改写选中文字</TooltipContent>
                        </Tooltip>
                      </>
                    )}
                    <Tooltip>
                      <TooltipTrigger
                        render={
                          <Button
                            variant="ghost"
                            size="xs"
                            onClick={() => {
                              setActiveMode("chat");
                              if (!aiPanelOpen) setAiPanelOpen(true);
                            }}
                            data-icon="inline-start"
                            className="text-xs"
                          >
                            <PenLine />
                            对话
                          </Button>
                        }
                      />
                      <TooltipContent>AI 对话</TooltipContent>
                    </Tooltip>
                  </div>
                </BubbleMenu>
              )}
              <EditorContent editor={editor} />
              {editor && aiDiffActive && (
                <AiFloatingPanel
                  editor={editor}
                  onCancel={handleReject}
                />
              )}
              {editor && aiDiffActive && !isStreaming && actionBarPos && (
                <div
                  className="absolute z-50 flex items-center gap-1 px-1.5 py-1 rounded-lg border border-border bg-popover shadow-lg animate-fade-in"
                  style={{ top: actionBarPos.top, left: actionBarPos.left }}
                  data-diff-action
                >
                  <button
                    className="flex items-center gap-1 text-xs font-medium px-2.5 py-1 rounded-md hover:bg-[oklch(0.7_0.16_160/0.12)] text-[oklch(0.75_0.16_160)] cursor-pointer transition-colors"
                    onMouseDown={(e) => { e.preventDefault(); handleAccept(); }}
                  >
                    <Check className="size-3" />
                    接受
                  </button>
                  <span className="text-muted-foreground/30 text-[10px]">|</span>
                  <button
                    className="flex items-center gap-1 text-xs font-medium px-2.5 py-1 rounded-md hover:bg-muted text-muted-foreground cursor-pointer transition-colors"
                    onMouseDown={(e) => { e.preventDefault(); handleReject(); }}
                  >
                    <X className="size-3" />
                    拒绝
                  </button>
                </div>
              )}
            </div>
          </div>

          <div className="flex items-center justify-between border-t border-border px-5 h-7 shrink-0">
            <div className="flex items-center gap-3 text-[11px] text-muted-foreground">
              <span className="flex items-center gap-1">
                <FileText className="size-3" />
                {wordCount} 字
              </span>
              {saving && (
                <span className="flex items-center gap-1 text-primary/60">
                  <Save className="size-3 animate-pulse-subtle" />
                  保存中
                </span>
              )}
            </div>
            <div className="flex items-center gap-2">
              <Select value={status} onValueChange={handleStatusChange}>
                <SelectTrigger size="sm" className="w-auto h-6 gap-1.5 px-2 text-[11px]">
                  <SelectValue placeholder="状态">{statusConfig[status].label}</SelectValue>
                </SelectTrigger>
                <SelectContent side="top">
                  <SelectGroup>
                    {Object.entries(statusConfig).map(([key, { label }]) => (
                      <SelectItem key={key} value={key}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
              <span className={cn("size-1.5 rounded-full", statusConfig[status].dot)} />
            </div>
          </div>
        </div>

        <AnimatePresence>
          {aiPanelOpen && (
            <AiPanel
              open={aiPanelOpen}
              onClose={() => setAiPanelOpen(false)}
            />
          )}
        </AnimatePresence>
      </div>
    </div>
  );
};

export default Editor;
