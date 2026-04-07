import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  Project,
  OutlineNode,
  Character,
  WorldviewEntry,
  AiConfig,
  AiAgent,
  StreamChunk,
  NarrativeSession,
  NarrativeBeat,
  NarrativeEvent,
  NarrativeStreamChunk,
  CharacterRelation,
  CharacterFaction,
  AggregateReview,
  WritingReview,
  Workflow,
  WorkflowStep,
  WorkflowProgress,
  WorkflowResult,
  WorkflowStepType,
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
  generateVolumeStructure: (projectId: string, concept: string, agentId: string) =>
    invoke<OutlineNode[]>("generate_volume_structure", { projectId, concept, agentId }),
  generateChapterStructure: (volumeId: string, agentId: string) =>
    invoke<OutlineNode[]>("generate_chapter_structure", { volumeId, agentId }),
  expandChapterOutline: (chapterId: string, agentId: string) =>
    invoke<OutlineNode>("expand_chapter_outline", { chapterId, agentId }),
};

export const characterApi = {
  list: (projectId: string) =>
    invoke<Character[]>("list_characters", { projectId }),
  get: (id: string) => invoke<Character>("get_character", { id }),
  create: (
    projectId: string,
    name: string,
    aliases: unknown[] | null,
    description: string,
    personality: string,
    background: string,
    race: string,
    modelId?: string | null,
  ) =>
    invoke<Character>("create_character", {
      projectId,
      name,
      aliases: aliases ?? [],
      description,
      personality,
      background,
      race,
      modelId: modelId ?? null,
    }),
  update: (
    id: string,
    name: string,
    aliases: unknown[] | null,
    description: string,
    personality: string,
    background: string,
    race: string,
    modelId?: string | null,
  ) =>
    invoke<Character>("update_character", {
      id,
      name,
      aliases: aliases ?? [],
      description,
      personality,
      background,
      race,
      modelId: modelId ?? null,
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
  listAgents: () => invoke<AiAgent[]>("list_ai_agents"),
  createAgent: (name: string, modelId: string | null, systemPrompt: string, temperature?: number | null) =>
    invoke<AiAgent>("create_ai_agent", { name, modelId, systemPrompt, temperature: temperature ?? null }),
  updateAgent: (id: string, name: string, modelId: string | null, systemPrompt: string, temperature?: number | null) =>
    invoke<AiAgent>("update_ai_agent", { id, name, modelId, systemPrompt, temperature: temperature ?? null }),
  deleteAgent: (id: string) => invoke<void>("delete_ai_agent", { id }),
  setDefaultAgent: (id: string) => invoke<void>("set_default_ai_agent", { id }),
  invoke: (agentId: string, userText: string) =>
    invoke<string>("ai_invoke", { agentId, userText }),
  invokeWithContext: (
    agentId: string,
    projectId: string,
    userText: string,
    onChunk: (chunk: StreamChunk) => void,
  ): Promise<void> => {
    const channel = new Channel<StreamChunk>(onChunk);
    return invoke("ai_invoke_with_context", {
      agentId,
      projectId,
      userText,
      onChunk: channel,
    });
  },
  stream: (
    params: {
      projectId: string;
      agentId: string;
      text: string;
      saveHistory?: boolean;
    },
    onChunk: (chunk: StreamChunk) => void,
  ): Promise<void> => {
    const channel = new Channel<StreamChunk>(onChunk);
    return invoke("ai_stream", {
      ...params,
      saveHistory: params.saveHistory ?? false,
      onChunk: channel,
    });
  },
  getChatHistory: (projectId: string) =>
    invoke<Array<{ role: string; content: string }>>("get_chat_history", { projectId }),
  clearChatHistory: (projectId: string) =>
    invoke<void>("clear_chat_history", { projectId }),
};

export const narrativeApi = {
  listSessions: (projectId: string) =>
    invoke<NarrativeSession[]>("list_narrative_sessions", { projectId }),
  getSession: (id: string) =>
    invoke<NarrativeSession>("get_narrative_session", { id }),
  createSession: (projectId: string, title: string, scene: string) =>
    invoke<NarrativeSession>("create_narrative_session", { projectId, title, scene }),
  deleteSession: (id: string) =>
    invoke<void>("delete_narrative_session", { id }),
  listBeats: (sessionId: string) =>
    invoke<NarrativeBeat[]>("list_narrative_beats", { sessionId }),
  listEvents: (sessionId: string) =>
    invoke<NarrativeEvent[]>("list_narrative_events", { sessionId }),
  deleteBeat: (id: string) =>
    invoke<void>("delete_narrative_beat", { id }),
  addBeat: (
    sessionId: string,
    beatType: string,
    characterId: string | null,
    characterName: string,
    content: string,
    strand?: string | null,
  ) =>
    invoke<NarrativeBeat>("add_narrative_beat", {
      sessionId,
      beatType,
      input: { characterId, characterName, content, strand },
    }),
  advanceNarration: (
    sessionId: string,
    onChunk: (chunk: NarrativeStreamChunk) => void,
    agentId?: string | null,
    instruction?: string | null,
  ): Promise<void> => {
    const channel = new Channel<NarrativeStreamChunk>(onChunk);
    return invoke("advance_narration", {
      sessionId,
      agentId: agentId ?? null,
      instruction: instruction ?? null,
      onChunk: channel,
    });
  },
  invokeCharacter: (
    sessionId: string,
    characterId: string,
    onChunk: (chunk: NarrativeStreamChunk) => void,
    agentId?: string | null,
    instruction?: string | null,
  ): Promise<void> => {
    const channel = new Channel<NarrativeStreamChunk>(onChunk);
    return invoke("invoke_narrative_character", {
      sessionId,
      characterId,
      agentId: agentId ?? null,
      instruction: instruction ?? null,
      onChunk: channel,
    });
  },
};

export const relationApi = {
  listRelations: (projectId: string) =>
    invoke<CharacterRelation[]>("list_character_relations", { projectId }),
  createRelation: (
    projectId: string,
    charAId: string,
    charBId: string,
    relationshipType: string,
    description: string,
    startChapterId?: string | null,
    endChapterId?: string | null,
  ) =>
    invoke<CharacterRelation>("create_character_relation", {
      projectId,
      charAId,
      charBId,
      relationshipType,
      description,
      startChapterId: startChapterId ?? null,
      endChapterId: endChapterId ?? null,
    }),
  updateRelation: (
    id: string,
    relationshipType: string,
    description: string,
    startChapterId?: string | null,
    endChapterId?: string | null,
  ) =>
    invoke<CharacterRelation>("update_character_relation", {
      id,
      relationshipType,
      description,
      startChapterId: startChapterId ?? null,
      endChapterId: endChapterId ?? null,
    }),
  deleteRelation: (id: string) =>
    invoke<void>("delete_character_relation", { id }),
  listFactions: (projectId: string) =>
    invoke<CharacterFaction[]>("list_character_factions", { projectId }),
  createFaction: (
    projectId: string,
    characterId: string,
    faction: string,
    role: string,
    startChapterId?: string | null,
    endChapterId?: string | null,
  ) =>
    invoke<CharacterFaction>("create_character_faction", {
      projectId,
      characterId,
      faction,
      role,
      startChapterId: startChapterId ?? null,
      endChapterId: endChapterId ?? null,
    }),
  updateFaction: (
    id: string,
    faction: string,
    role: string,
    startChapterId?: string | null,
    endChapterId?: string | null,
  ) =>
    invoke<CharacterFaction>("update_character_faction", {
      id,
      faction,
      role,
      startChapterId: startChapterId ?? null,
      endChapterId: endChapterId ?? null,
    }),
  deleteFaction: (id: string) =>
    invoke<void>("delete_character_faction", { id }),
};

export const reviewApi = {
  reviewBeat: (
    sessionId: string,
    beatId: string,
    beatContent: string,
    beatType: string,
    agentId?: string | null,
  ) =>
    invoke<AggregateReview>("review_beat", {
      sessionId,
      beatId,
      beatContent,
      beatType,
      agentId: agentId ?? null,
    }),
  listReviews: (sessionId: string) =>
    invoke<WritingReview[]>("list_writing_reviews", { sessionId }),
};

export const workflowApi = {
  list: () => invoke<Workflow[]>("list_workflows"),
  getSteps: (workflowId: string) =>
    invoke<WorkflowStep[]>("list_workflow_steps", { workflowId }),
  get: (id: string) => invoke<Workflow>("get_workflow", { id }),
  create: (
    name: string,
    description: string,
    steps: Array<{
      step_type: WorkflowStepType;
      agent_id: string | null;
      condition: Record<string, unknown> | null;
      config: Record<string, unknown>;
      enabled: boolean;
    }>,
  ) => invoke<Workflow>("create_workflow", { name, description, steps }),
  update: (
    id: string,
    name: string,
    description: string,
    steps: Array<{
      step_type: WorkflowStepType;
      agent_id: string | null;
      condition: Record<string, unknown> | null;
      config: Record<string, unknown>;
      enabled: boolean;
    }>,
  ) => invoke<Workflow>("update_workflow", { id, name, description, steps }),
  delete: (id: string) => invoke<void>("delete_workflow", { id }),
  setDefault: (id: string) => invoke<void>("set_default_workflow", { id }),
  run: (
    workflowId: string,
    params: {
      projectId: string;
      sessionId?: string | null;
      text?: string | null;
      instruction?: string | null;
      characterId?: string | null;
    },
    onProgress: (chunk: WorkflowProgress) => void,
  ): Promise<WorkflowResult> => {
    const channel = new Channel<WorkflowProgress>(onProgress);
    return invoke("run_workflow", {
      workflowId,
      projectId: params.projectId,
      sessionId: params.sessionId ?? null,
      text: params.text ?? null,
      instruction: params.instruction ?? null,
      characterId: params.characterId ?? null,
      onProgress: channel,
    });
  },
};
