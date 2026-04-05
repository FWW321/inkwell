import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  Project,
  OutlineNode,
  Character,
  WorldviewEntry,
  AiConfig,
  StreamChunk,
} from "./types";

export const projectApi = {
  list: () => invoke<Project[]>("list_projects"),
  get: (id: string) => invoke<Project>("get_project", { id }),
  create: (title: string, description: string, author: string, language: string, tags: string, status: string) =>
    invoke<Project>("create_project", { title, description, author, language, tags, status }),
  update: (id: string, title: string, description: string, author: string, language: string, tags: string, status: string) =>
    invoke<Project>("update_project", { id, title, description, author, language, tags, status }),
  delete: (id: string) => invoke<void>("delete_project", { id }),
};

export const outlineApi = {
  list: (projectId: string, parentId?: string | null) =>
    invoke<OutlineNode[]>("list_outline_nodes", { projectId, parentId: parentId ?? null }),
  get: (id: string) => invoke<OutlineNode>("get_outline_node", { id }),
  create: (projectId: string, parentId: string | null, nodeType: string, title: string) =>
    invoke<OutlineNode>("create_outline_node", { projectId, parentId, nodeType, title }),
  update: (id: string, title: string, contentJson: string, wordCount: number, status: string) =>
    invoke<OutlineNode>("update_outline_node", { id, title, contentJson, wordCount, status }),
  rename: (id: string, title: string) =>
    invoke<OutlineNode>("rename_outline_node", { id, title }),
  delete: (id: string) => invoke<void>("delete_outline_node", { id }),
  reorder: (projectId: string, parentId: string | null, nodeIds: string[]) =>
    invoke<void>("reorder_outline_nodes", { projectId, parentId, nodeIds }),
  saveDiff: (id: string, originalText: string, newText: string, mode: string) =>
    invoke<void>("save_diff", { id, originalText, newText, mode }),
  clearDiff: (id: string) =>
    invoke<void>("clear_diff", { id }),
};

export const characterApi = {
  list: (projectId: string) =>
    invoke<Character[]>("list_characters", { projectId }),
  get: (id: string) => invoke<Character>("get_character", { id }),
  create: (
    projectId: string,
    name: string,
    description: string,
    personality: string,
    background: string,
  ) =>
    invoke<Character>("create_character", {
      projectId,
      name,
      description,
      personality,
      background,
    }),
  update: (
    id: string,
    name: string,
    description: string,
    personality: string,
    background: string,
  ) =>
    invoke<Character>("update_character", {
      id,
      name,
      description,
      personality,
      background,
    }),
  delete: (id: string) => invoke<void>("delete_character", { id }),
};

export const worldviewApi = {
  list: (projectId: string) =>
    invoke<WorldviewEntry[]>("list_worldview_entries", { projectId }),
  create: (projectId: string, category: string, title: string, content: string) =>
    invoke<WorldviewEntry>("create_worldview_entry", {
      projectId,
      category,
      title,
      content,
    }),
  update: (id: string, category: string, title: string, content: string) =>
    invoke<WorldviewEntry>("update_worldview_entry", {
      id,
      category,
      title,
      content,
    }),
  delete: (id: string) => invoke<void>("delete_worldview_entry", { id }),
};

export const aiApi = {
  listModels: () => invoke<AiConfig[]>("list_ai_models"),
  createModel: (name: string, apiKey: string, model: string, baseUrl: string) =>
    invoke<AiConfig>("create_ai_model", { name, apiKey, model, baseUrl }),
  updateModel: (id: string, name: string, apiKey: string, model: string, baseUrl: string) =>
    invoke<AiConfig>("update_ai_model", { id, name, apiKey, model, baseUrl }),
  deleteModel: (id: string) => invoke<void>("delete_ai_model", { id }),
  setDefault: (id: string) => invoke<void>("set_default_ai_model", { id }),
  fetchAvailableModels: (apiKey?: string, baseUrl?: string) =>
    invoke<string[]>("list_models", { apiKey, baseUrl }),
  continueWriting: (context: string, style: string, length: string) =>
    invoke<string>("ai_continue_writing", { context, style, length }),
  rewrite: (selectedText: string, instruction: string) =>
    invoke<string>("ai_rewrite", { selectedText, instruction }),
  polish: (selectedText: string) =>
    invoke<string>("ai_polish", { selectedText }),
  generateDialogue: (characters: string, scenario: string) =>
    invoke<string>("ai_generate_dialogue", { characters, scenario }),
  chat: (projectId: string, contextType: string, contextId: string, message: string) =>
    invoke<string>("ai_chat", { projectId, contextType, contextId, message }),
  stream: (
    params: {
      projectId: string;
      chapterId?: string | null;
      mode: string;
      text: string;
      style?: string | null;
      length?: string | null;
    },
    onChunk: (chunk: StreamChunk) => void,
  ): Promise<void> => {
    const channel = new Channel<StreamChunk>(onChunk);
    return invoke("ai_stream", {
      ...params,
      onChunk: channel,
    });
  },
  getChatHistory: (projectId: string) =>
    invoke<Array<{ role: string; content: string }>>("get_chat_history", { projectId }),
  clearChatHistory: (projectId: string) =>
    invoke<void>("clear_chat_history", { projectId }),
};
