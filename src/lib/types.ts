export type ProjectStatus = "ongoing" | "completed" | "hiatus";

export interface Project {
  id: string;
  title: string;
  description: string;
  author: string;
  language: string;
  tags: string;
  status: ProjectStatus;
  cover_url: string | null;
  created_at: string;
  updated_at: string;
}

export type NodeType = "volume" | "chapter";
export type ChapterStatus = "draft" | "completed" | "revising";

export interface OutlineNode {
  id: string;
  project_id: string;
  parent_id: string | null;
  node_type: NodeType;
  title: string;
  sort_order: number;
  content_json: string;
  word_count: number;
  status: ChapterStatus;
  diff_original?: string | null;
  diff_new?: string | null;
  diff_mode?: string | null;
  created_at: string;
  updated_at: string;
}

export interface Character {
  id: string;
  project_id: string;
  name: string;
  avatar_url: string | null;
  description: string;
  personality: string;
  background: string;
  created_at: string;
}

export interface WorldviewEntry {
  id: string;
  project_id: string;
  category: string;
  title: string;
  content: string;
  created_at: string;
}

export interface AiConfig {
  id: string;
  name: string;
  api_key: string;
  model: string;
  base_url: string;
  is_default: boolean;
  created_at: string;
}

export interface AiMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

export type AiMode = "continue" | "rewrite" | "polish" | "dialogue" | "chat";

export interface StreamChunk {
  text: string;
  done: boolean;
  reasoning?: string;
}

export interface AiEditorState {
  selectedText: string;
  cursorBefore: string;
  cursorAfter: string;
  chapterId: string;
  projectId: string;
  hasSelection: boolean;
}
