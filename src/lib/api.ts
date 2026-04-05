import { invoke } from "@tauri-apps/api/core";
import type {
  Project,
  OutlineNode,
  Character,
  WorldviewEntry,
  AiConfig,
} from "./types";

export const projectApi = {
  list: () => invoke<Project[]>("list_projects"),
  get: (id: string) => invoke<Project>("get_project", { id }),
  create: (title: string, description: string) =>
    invoke<Project>("create_project", { title, description }),
  update: (id: string, title: string, description: string) =>
    invoke<Project>("update_project", { id, title, description }),
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
  getConfig: () => invoke<AiConfig>("get_ai_config"),
  setConfig: (apiKey: string, model: string, baseUrl: string) =>
    invoke<void>("set_ai_config", { apiKey, model, baseUrl }),
  listModels: (apiKey?: string, baseUrl?: string) =>
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
};
