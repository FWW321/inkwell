export type ProjectStatus = "ongoing" | "completed" | "hiatus";

export type NodeType = "volume" | "chapter";
export type ChapterStatus = "draft" | "in_progress" | "complete" | "revised";

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
  aliases: unknown[] | null;
  avatar_url: string | null;
  description: string;
  personality: string;
  background: string;
  race: string;
  model_id: string | null;
  model_name: string | null;
  created_at: string;
}

export interface WorldviewEntry {
  id: string;
  project_id: string;
  category: string;
  title: string;
  content: string;
  created_at: string;
  updated_at: string | null;
}

export interface IpcError {
  code: string;
  message: string;
}

export interface AiConfig {
  id: string;
  name: string;
  model: string;
  base_url: string;
  is_default: boolean;
  created_at: string;
}

export interface AiAgent {
  id: string;
  name: string;
  model_id: string | null;
  system_prompt: string;
  temperature: number;
  is_default: boolean;
  created_at: string;
  model_name: string | null;
}

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
  timeline_id: string;
  strand: "quest" | "fire" | "constellation";
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
  timeline_id: string;
  strand: "quest" | "fire" | "constellation";
  hook_type?: string | null;
  hook_strength?: string | null;
  micro_payoffs: unknown[] | null;
  created_at: string;
}

export interface NarrativeEvent {
  id: string;
  beat_id: string;
  session_id: string;
  event_type: string;
  character_id: string | null;
  character_name: string;
  summary: string;
  detail: Record<string, unknown>;
  created_at: string;
}

export interface NarrativeStreamChunk {
  beat_id: string;
  beat_type: string;
  strand: string;
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

export type Strand = "quest" | "fire" | "constellation";

export interface WritingReview {
  id: string;
  session_id: string;
  beat_id: string;
  dimension: string;
  score: number;
  passed: boolean;
  issues: ReviewIssue[];
  summary: string;
  created_at: string;
}

export interface ReviewIssue {
  severity: "high" | "medium" | "low";
  description: string;
  suggestion: string;
}

export interface AggregateReview {
  dimensions: {
    dimension: string;
    score: number;
    passed: boolean;
    issues: ReviewIssue[];
    summary: string;
  }[];
  overall_score: number;
  passed: boolean;
}

export type WorkflowStepType =
  | "generate_worldview"
  | "generate_characters"
  | "generate_volume_structure"
  | "generate_chapter_structure"
  | "expand_chapter_outline"
  | "narrate"
  | "character_action"
  | "polish"
  | "rewrite"
  | "continue_writing"
  | "dialogue"
  | "review";

export interface WorkflowStep {
  id: string;
  workflow_id: string;
  sort_order: number;
  step_type: WorkflowStepType;
  agent_id: string | null;
  condition: Record<string, unknown> | null;
  config: Record<string, unknown>;
  enabled: boolean;
}

export interface Workflow {
  id: string;
  name: string;
  description: string;
  is_preset: boolean;
  is_default: boolean;
  step_count: number;
  created_at: string;
  updated_at: string;
}

export interface WorkflowProgress {
  step_index: number;
  step_count: number;
  step_type: string;
  step_label: string;
  status: "running" | "completed" | "skipped" | "error" | "done";
  message: string;
  text: string;
  done: boolean;
}

export interface WorkflowStepResult {
  step_index: number;
  type_str: string;
  label: string;
  status: "completed" | "skipped" | "error";
  output_text: string;
  score: number | null;
  error: string | null;
}

export interface WorkflowResult {
  steps: WorkflowStepResult[];
  final_text: string;
}

export interface WorkflowStepTypeOption {
  value: WorkflowStepType;
  label: string;
}
