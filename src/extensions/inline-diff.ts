import { Extension, type Command } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

export const inlineDiffPluginKey = new PluginKey("inkwellInlineDiff");

let diffText = "";
let diffReady = false;

export function setDiffText(text: string) {
  diffText = text;
  const el = document.querySelector("[data-inline-diff='new']") as HTMLElement | null;
  if (el && el.isConnected) el.textContent = text;
}

export function getDiffText(): string {
  return diffText;
}

export function setDiffReady(ready: boolean) {
  if (ready === diffReady) return;
  diffReady = ready;
}

export function clearDiffText() {
  diffText = "";
  diffReady = false;
}

interface DiffState {
  from: number;
  to: number;
}

const emptyState: DiffState = { from: 0, to: 0 };

export const InlineDiff = Extension.create({
  name: "inlineDiff",

  addCommands() {
    return {
      setInlineDiff:
        (from: number, to: number): Command =>
        ({ tr, dispatch }) => {
          if (dispatch) {
            dispatch(tr.setMeta(inlineDiffPluginKey, { from, to }));
          }
          return true;
        },
      clearInlineDiff:
        (): Command =>
        ({ tr, dispatch }) => {
          if (dispatch) {
            dispatch(tr.setMeta(inlineDiffPluginKey, { clear: true }));
          }
          return true;
        },
    } as any;
  },

  addProseMirrorPlugins() {
    return [
      new Plugin<DiffState>({
        key: inlineDiffPluginKey,
        state: {
          init: () => emptyState,
          apply(tr, prev) {
            const meta = tr.getMeta(inlineDiffPluginKey);
            if (meta) {
              if (meta.clear) {
                clearDiffText();
                return emptyState;
              }
              if (meta.from !== undefined) {
                return { from: meta.from, to: meta.to };
              }
            }
            if (tr.docChanged && prev.from !== prev.to) {
              let overlaps = false;
              for (const step of tr.steps) {
                const sf = (step as any).from as number;
                const st = (step as any).to as number;
                if (sf < prev.to && st > prev.from) {
                  overlaps = true;
                  break;
                }
              }
              if (overlaps) {
                clearDiffText();
                return emptyState;
              }

              let atEnd = false;
              for (const step of tr.steps) {
                const sf = (step as any).from as number;
                if (sf >= prev.to) {
                  atEnd = true;
                  break;
                }
              }
              if (atEnd) return prev;

              return {
                from: tr.mapping.map(prev.from),
                to: tr.mapping.map(prev.to),
              };
            }
            return prev;
          },
        },
        props: {
          decorations(state) {
            const gs = this.getState(state);
            if (!gs || gs.from === gs.to) return null;
            const cls = diffReady ? "inline-diff-original inline-diff-ready" : "inline-diff-original inline-diff-streaming";
            return DecorationSet.create(state.doc, [
              Decoration.inline(gs.from, gs.to, {
                class: cls,
                "data-inline-diff": "original",
              }),
              Decoration.widget(
                gs.to,
                () => {
                  const span = document.createElement("span");
                  span.className = "inline-diff-new";
                  span.contentEditable = "false";
                  span.setAttribute("data-inline-diff", "new");
                  span.textContent = diffText;
                  return span;
                },
                {
                  side: -1,
                  key: "inkwell-diff-new",
                  stopEvent: () => true,
                },
              ),
            ]);
          },
        },
      }),
    ];
  },
});
