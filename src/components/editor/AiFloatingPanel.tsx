import { useState, useEffect, useCallback, useRef } from "react";
import { AnimatePresence, motion } from "motion/react";
import { Brain, ChevronDown, X } from "lucide-react";
import type { Editor } from "@tiptap/react";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useAiEditor } from "@/contexts/AiEditorContext";
import { springs } from "@/lib/motion";

interface AiFloatingPanelProps {
  editor: Editor;
  onCancel: () => void;
}

export default function AiFloatingPanel({ editor, onCancel }: AiFloatingPanelProps) {
  const { isStreaming, streamingReasoning } = useAiEditor();
  const [reasoningCollapsed, setReasoningCollapsed] = useState(true);
  const [pos, setPos] = useState<{ top: number; left: number } | null>(null);
  const mountedRef = useRef(true);

  const updatePosition = useCallback(() => {
    if (!mountedRef.current) return;

    const parent = editor.view.dom.closest(".relative") as HTMLElement | null;
    if (!parent) { setPos(null); return; }
    const parentRect = parent.getBoundingClientRect();

    const widget = editor.view.dom.querySelector("[data-inline-diff='new']") as HTMLElement | null;
    const original = editor.view.dom.querySelector("[data-inline-diff='original']") as HTMLElement | null;
    const target = widget || original;
    if (!target) { setPos(null); return; }

    const rect = target.getBoundingClientRect();
    const top = rect.bottom - parentRect.top + 6;
    const left = Math.max(8, rect.left - parentRect.left);

    setPos({ top, left });
  }, [editor]);

  useEffect(() => {
    mountedRef.current = true;
    updatePosition();

    const parent = editor.view.dom.closest(".relative") as HTMLElement | null;
    const scrollParent = parent?.closest(".overflow-y-auto") as HTMLElement | null;

    const onScroll = () => requestAnimationFrame(updatePosition);
    scrollParent?.addEventListener("scroll", onScroll, { passive: true });

    const observer = new MutationObserver(() => requestAnimationFrame(updatePosition));
    observer.observe(editor.view.dom, { childList: true, subtree: true, characterData: true });

    const timer = setInterval(updatePosition, 500);

    return () => {
      mountedRef.current = false;
      clearInterval(timer);
      scrollParent?.removeEventListener("scroll", onScroll);
      observer.disconnect();
    };
  }, [editor, updatePosition]);

  const visible = isStreaming || streamingReasoning;

  return (
    <AnimatePresence>
      {pos && visible && (
        <motion.div
          key="ai-floating"
          className="absolute z-40"
          style={{ top: pos.top, left: pos.left }}
          initial={{ opacity: 0, scale: 0.95, y: 4 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: 4 }}
          transition={springs.gentle}
        >
          <div className="flex flex-col gap-0.5 rounded-lg border border-border bg-popover/95 backdrop-blur-md shadow-lg overflow-hidden">
            {streamingReasoning && (
              <>
                <button
                  onClick={() => setReasoningCollapsed(!reasoningCollapsed)}
                  className="flex items-center gap-1.5 px-3 py-1.5 text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors"
                >
                  <Brain className="size-3 shrink-0" />
                  <span className="flex-1 text-left truncate">
                    {reasoningCollapsed ? "AI 思考中..." : `思考 (${streamingReasoning.length}字)`}
                  </span>
                  {!reasoningCollapsed && <Spinner className="size-2.5 shrink-0 text-primary/50" />}
                  <ChevronDown className={`size-2.5 shrink-0 transition-transform ${reasoningCollapsed ? "" : "rotate-180"}`} />
                </button>
                <AnimatePresence initial={false}>
                  {!reasoningCollapsed && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.2 }}
                      className="overflow-hidden"
                    >
                      <div className="max-h-28 overflow-y-auto px-3 pb-2 border-b border-border">
                        <p className="text-xs leading-relaxed text-muted-foreground/70 whitespace-pre-wrap">
                          {streamingReasoning}
                          {isStreaming && (
                            <span className="inline-block size-1 ml-0.5 rounded-full bg-primary/40 animate-pulse" />
                          )}
                        </p>
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>
              </>
            )}
            <div className="flex items-center justify-end gap-1 px-2 py-1.5">
              <div className="flex items-center gap-1.5 text-xs text-muted-foreground mr-1">
                <Spinner className="size-3 text-primary/60" />
                <span>AI 处理中</span>
              </div>
              <Button
                variant="ghost"
                size="xs"
                onClick={onCancel}
                className="text-xs text-muted-foreground hover:text-destructive"
              >
                <X className="size-3" />
                取消
              </Button>
            </div>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
