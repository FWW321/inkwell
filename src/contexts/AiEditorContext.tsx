import { createContext, useContext, useState, useCallback, useRef } from "react";
import type { ReactNode } from "react";
import type { Editor } from "@tiptap/react";
import type { AiEditorState, StreamChunk, AiAgent } from "@/lib/types";
import { aiApi } from "@/lib/api";
import { setDiffText } from "@/extensions/inline-diff";

export type EditorMode = "polish" | "rewrite" | "dialogue" | "chat";

const MODE_AGENT_NAMES: Record<EditorMode, string> = {
  polish: "润色助手",
  rewrite: "改写助手",
  dialogue: "对话生成",
  chat: "写作顾问",
};

interface AiEditorContextValue {
  editorState: AiEditorState;
  setProjectInfo: (projectId: string, chapterId: string) => void;
  refreshCursor: () => void;

  isStreaming: boolean;
  streamingText: string;
  streamingReasoning: string;
  generatedText: string;
  activeMode: EditorMode;

  startStreaming: (params: {
    text: string;
    instruction?: string;
  }) => Promise<void>;
  stopStreaming: () => void;
  setGeneratedText: (text: string) => void;
  setActiveMode: (mode: EditorMode) => void;
  replaceSelection: (text: string) => void;
  insertAtCursor: (text: string) => void;
  getEditor: () => Editor | null;
  setEditorRef: (editor: Editor) => void;
}

const AiEditorContext = createContext<AiEditorContextValue | null>(null);

export function AiEditorProvider({ children }: { children: ReactNode }) {
  const [editorState, setEditorState] = useState<AiEditorState>({
    selectedText: "",
    cursorBefore: "",
    cursorAfter: "",
    chapterId: "",
    projectId: "",
    hasSelection: false,
  });

  const [isStreaming, setIsStreaming] = useState(false);
  const [streamingText, setStreamingText] = useState("");
  const [streamingReasoning, setStreamingReasoning] = useState("");
  const [generatedText, setGeneratedText] = useState("");
  const [activeMode, setActiveMode] = useState<EditorMode>("polish");
  const [agents, setAgents] = useState<AiAgent[]>([]);
  const editorRef = useRef<Editor | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const setProjectInfo = useCallback((projectId: string, chapterId: string) => {
    setEditorState((prev) => ({ ...prev, projectId, chapterId }));
  }, []);

  const refreshCursor = useCallback(() => {
    setEditorState((prev) => {
      if (!editorRef.current) return prev;
      const { from, to, empty } = editorRef.current.state.selection;
      const hasSelection = !empty && from !== to;
      const selectedText = hasSelection
        ? editorRef.current.state.doc.textBetween(from, to, "\n")
        : "";
      return { ...prev, selectedText, hasSelection };
    });
  }, []);

  const resolveAgentId = useCallback((mode: EditorMode): string => {
    const targetName = MODE_AGENT_NAMES[mode];
    const match = agents.find((a) => a.name === targetName);
    if (match) return match.id;
    return agents[0]?.id ?? "";
  }, [agents]);

  const loadAgents = useCallback(async () => {
    try {
      const list = await aiApi.listAgents();
      setAgents(list);
    } catch {}
  }, []);

  useState(() => { loadAgents(); });

  const streamRef = useRef("");
  const reasoningRef = useRef("");

  const startStreaming = useCallback(
    async (params: {
      text: string;
      instruction?: string;
    }) => {
      if (!editorRef.current) return;
      const currentState = { ...editorState };
      const agentId = resolveAgentId(activeMode);

      let promptText = params.text;
      if (activeMode === "polish" && !promptText.trim()) {
        promptText = currentState.selectedText || "";
      }

      if (params.instruction) {
        promptText = `[指令: ${params.instruction}]\n\n${promptText}`;
      }

      setIsStreaming(true);
      setStreamingText("");
      setStreamingReasoning("");
      setGeneratedText("");
      streamRef.current = "";
      reasoningRef.current = "";
      abortRef.current = new AbortController();

      const handleChunk = (chunk: StreamChunk) => {
        if (abortRef.current?.signal.aborted) return;
        if (chunk.reasoning) {
          reasoningRef.current += chunk.reasoning;
          setStreamingReasoning(reasoningRef.current);
        }
        if (chunk.text) {
          streamRef.current += chunk.text;
          setStreamingText(streamRef.current);
          if (activeMode !== "chat") {
            setDiffText(streamRef.current);
          }
        }
        if (chunk.done) {
          setIsStreaming(false);
          setGeneratedText(streamRef.current);
          setStreamingReasoning("");
          reasoningRef.current = "";
        }
      };

      try {
        if (activeMode === "chat") {
          await aiApi.stream(
            {
              projectId: currentState.projectId,
              agentId,
              text: promptText,
              saveHistory: true,
            },
            handleChunk,
          );
        } else {
          await aiApi.invokeWithContext(
            agentId,
            currentState.projectId,
            promptText,
            handleChunk,
          );
        }
      } catch {
        setIsStreaming(false);
      }
    },
    [editorState, activeMode, resolveAgentId],
  );

  const stopStreaming = useCallback(() => {
    abortRef.current?.abort();
    setIsStreaming(false);
  }, []);

  const replaceSelection = useCallback((text: string) => {
    if (!editorRef.current) return;
    const editor = editorRef.current;
    const { from, to, empty } = editor.state.selection;

    if (!empty && from !== to) {
      editor.chain().focus().deleteSelection().insertContent(text).run();
    } else {
      const lines = text.split("\n");
      editor.chain().focus().run();
      for (const line of lines) {
        editor.chain().focus().createParagraphNear().insertContent(line).run();
      }
    }
  }, []);

  const insertAtCursor = useCallback((text: string) => {
    if (!editorRef.current) return;
    editorRef.current.chain().focus().insertContent(text).run();
  }, []);

  const setEditorRef = useCallback((editor: Editor) => {
    editorRef.current = editor;
  }, []);

  const getEditor = useCallback(() => editorRef.current, []);

  return (
    <AiEditorContext.Provider
      value={{
        editorState,
        setProjectInfo,
        refreshCursor,
        isStreaming,
        streamingText,
        streamingReasoning,
        generatedText,
        activeMode,
        startStreaming,
        stopStreaming,
        setGeneratedText,
        setActiveMode,
        replaceSelection,
        insertAtCursor,
        getEditor,
        setEditorRef,
      }}
    >
      {children}
    </AiEditorContext.Provider>
  );
}

export function useAiEditor(): AiEditorContextValue {
  const ctx = useContext(AiEditorContext);
  if (!ctx) {
    throw new Error("useAiEditor must be used within AiEditorProvider");
  }
  return ctx;
}
