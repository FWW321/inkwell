import { useState, useEffect, useCallback } from "react";
import type { Editor } from "@tiptap/react";
import type { AiEditorState } from "@/lib/types";

export function useEditorSelection(editor: Editor | null) {
  const [state, setState] = useState<AiEditorState>({
    selectedText: "",
    cursorBefore: "",
    cursorAfter: "",
    chapterId: "",
    projectId: "",
    hasSelection: false,
  });

  const updateFromEditor = useCallback(() => {
    if (!editor) return;

    const { from, to, empty } = editor.state.selection;

    let selectedText = "";
    let hasSelection = false;
    let cursorBefore = "";
    let cursorAfter = "";

    if (!empty && from !== to) {
      selectedText = editor.state.doc.textBetween(from, to, "\n");
      hasSelection = true;
    } else {
      const docSize = editor.state.doc.content.size;
      const cursorPos = from;

      const textBefore = editor.state.doc.textBetween(
        Math.max(0, cursorPos - 500),
        cursorPos,
        "\n",
      );
      const lastBreak = textBefore.lastIndexOf("\n");
      cursorBefore = lastBreak >= 0 ? textBefore.slice(lastBreak + 1) : textBefore;

      const textAfter = editor.state.doc.textBetween(
        cursorPos,
        Math.min(docSize, cursorPos + 500),
        "\n",
      );
      const nextBreak = textAfter.indexOf("\n");
      cursorAfter = nextBreak >= 0 ? textAfter.slice(0, nextBreak) : textAfter;
    }

    setState((prev) => ({
      ...prev,
      selectedText,
      hasSelection,
      cursorBefore,
      cursorAfter,
    }));
  }, [editor]);

  useEffect(() => {
    if (!editor) return;

    editor.on("selectionUpdate", updateFromEditor);
    editor.on("update", updateFromEditor);

    return () => {
      editor.off("selectionUpdate", updateFromEditor);
      editor.off("update", updateFromEditor);
    };
  }, [editor, updateFromEditor]);

  const setProjectInfo = useCallback((projectId: string, chapterId: string) => {
    setState((prev) => ({ ...prev, projectId, chapterId }));
  }, []);

  const refreshCursor = useCallback(() => {
    updateFromEditor();
  }, [updateFromEditor]);

  return { state, setProjectInfo, refreshCursor };
}
