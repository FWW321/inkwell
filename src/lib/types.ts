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
  content_json: Record<string, unknown>;
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
  race: string;
  model_id?: string | null;
  model_name?: string | null;
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

export interface AiAgent {
  id: string;
  name: string;
  model_id: string;
  system_prompt: string;
  is_default: boolean;
  created_at: string;
  model_name?: string | null;
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

export interface NarrativeSession {
  id: string;
  project_id: string;
  title: string;
  scene: string;
  atmosphere: string;
  character_states: Record<string, unknown>;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface NarrativeBeat {
  id: string;
  session_id: string;
  beat_type: "narration" | "character_action" | "scene_change" | "author_intervention";
  character_id: string | null;
  character_name: string;
  content: string;
  metadata: Record<string, unknown>;
  sort_order: number;
  created_at: string;
}

export interface NarrativeStreamChunk {
  beat_id: string;
  beat_type: string;
  character_id: string | null;
  character_name: string;
  text: string;
  done: boolean;
}

export interface CharacterRelation {
  id: string;
  project_id: string;
  char_a_id: string;
  char_a_name: string;
  char_b_id: string;
  char_b_name: string;
  relationship_type: string;
  description: string;
  start_chapter_id: string | null;
  start_chapter_title: string | null;
  end_chapter_id: string | null;
  end_chapter_title: string | null;
  created_at: string;
}

export interface CharacterFaction {
  id: string;
  project_id: string;
  character_id: string;
  character_name: string;
  faction_id: string;
  faction_name: string;
  role: string;
  start_chapter_id: string | null;
  start_chapter_title: string | null;
  end_chapter_id: string | null;
  end_chapter_title: string | null;
  created_at: string;
}
