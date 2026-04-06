import { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import {
  Plus,
  UserRound,
  Trash2,
  Pencil,
  X,
  Check,
  ChevronDown,
  Search,
  Cpu,
  Swords,
  Network,
  Users,
} from "lucide-react";
import { characterApi, aiApi, relationApi, worldviewApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { Character, AiConfig, CharacterRelation, CharacterFaction, WorldviewEntry } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from "@/components/ui/empty";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectTrigger, SelectContent, SelectGroup, SelectItem, SelectValue } from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import CharacterGraph from "@/components/character/CharacterGraph";

const CharacterCard = ({
  character,
  relations,
  factions,
  onDelete,
  onUpdate,
  models,
  raceEntries,
}: {
  character: Character;
  relations: CharacterRelation[];
  factions: CharacterFaction[];
  onDelete: () => void;
  onUpdate: (data: {
    name: string;
    description: string;
    personality: string;
    background: string;
    race: string;
    modelId?: string | null;
  }) => void;
  models: AiConfig[];
  raceEntries: WorldviewEntry[];
}) => {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(character.name);
  const [description, setDescription] = useState(character.description);
  const [personality, setPersonality] = useState(character.personality);
  const [background, setBackground] = useState(character.background);
  const [race, setRace] = useState(character.race);
  const [modelId, setModelId] = useState(character.model_id ?? "");
  const [expanded, setExpanded] = useState(false);

  const handleSave = async () => {
    await onUpdate({ name, description, personality, background, race, modelId: modelId || null });
    setEditing(false);
  };

  const handleCancel = () => {
    setName(character.name);
    setDescription(character.description);
    setPersonality(character.personality);
    setBackground(character.background);
    setRace(character.race);
    setModelId(character.model_id ?? "");
    setEditing(false);
  };

  const charRelations = relations.filter(
    (r) => r.char_a_id === character.id || r.char_b_id === character.id,
  );
  const charFactions = factions.filter((f) => f.character_id === character.id);

  const formatChapterRange = (startTitle: string | null, endTitle: string | null) => {
    if (!startTitle && !endTitle) return "";
    if (startTitle && endTitle) return `${startTitle} - ${endTitle}`;
    return startTitle ? `从「${startTitle}」起` : `至「${endTitle}」止`;
  };

  return (
    <Card className="transition-all duration-200 hover:ring-primary/20">
      <div
        className="flex items-center gap-3.5 cursor-pointer select-none"
        onClick={() => !editing && setExpanded(!expanded)}
      >
        <div className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-primary/12">
          <UserRound className="size-5 text-primary/80" />
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-semibold text-card-foreground truncate">
            {character.name}
          </p>
          <div className="flex items-center gap-2 mt-0.5 flex-wrap">
            {character.race && (
              <span className="shrink-0 text-xs text-muted-foreground/60">
                {character.race}
              </span>
            )}
            {character.description && (
              <p className="text-sm text-muted-foreground truncate">
                {character.description}
              </p>
            )}
            {character.model_name && (
              <span className="shrink-0 inline-flex items-center gap-1 text-xs text-muted-foreground/60">
                <Cpu className="size-3" />
                {character.model_name}
              </span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-1">
          {expanded && !editing && (
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={(e) => {
                e.stopPropagation();
                setEditing(true);
              }}
              className="opacity-0 group-hover/card:opacity-100"
            >
              <Pencil />
            </Button>
          )}
          {!editing && (
            <Button
              variant="ghost"
              size="icon-sm"
              onClick={(e) => {
                e.stopPropagation();
                setExpanded(!expanded);
              }}
              className={cn(expanded && "rotate-180")}
            >
              <ChevronDown />
            </Button>
          )}
        </div>
      </div>

      {expanded && (
        <CardContent className="flex flex-col gap-4 animate-fade-in border-t">
          {editing ? (
            <>
              <div className="flex gap-2">
                <div className="flex flex-col gap-1.5 flex-1">
                  <Label className="text-xs uppercase tracking-wider">姓名</Label>
                  <Input
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                  />
                </div>
                <div className="flex flex-col gap-1.5 w-32">
                  <Label className="text-xs uppercase tracking-wider">种族</Label>
                  <Input
                    value={race}
                    onChange={(e) => setRace(e.target.value)}
                    placeholder="种族"
                    list={`race-list-${character.id}`}
                  />
                  <datalist id={`race-list-${character.id}`}>
                    {raceEntries.map((e) => (
                      <option key={e.id} value={e.title} />
                    ))}
                  </datalist>
                </div>
              </div>
              <div className="flex flex-col gap-1.5">
                <Label className="text-xs uppercase tracking-wider">外貌描述</Label>
                <Textarea
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  rows={2}
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <Label className="text-xs uppercase tracking-wider">性格特点</Label>
                <Textarea
                  value={personality}
                  onChange={(e) => setPersonality(e.target.value)}
                  rows={2}
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <Label className="text-xs uppercase tracking-wider">背景故事</Label>
                <Textarea
                  value={background}
                  onChange={(e) => setBackground(e.target.value)}
                  rows={3}
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <Label className="text-xs uppercase tracking-wider">AI 模型</Label>
                <Select value={modelId} onValueChange={(v) => setModelId(v ?? "")}>
                  <SelectTrigger>
                    <SelectValue>
                      {modelId ? (models.find((m) => m.id === modelId)?.name ?? modelId) : "默认模型"}
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="">
                        默认模型
                      </SelectItem>
                      {models.map((m) => (
                        <SelectItem key={m.id} value={m.id}>
                          {m.name}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground/60 mt-0.5">
                  指定模型后，AI 将以该角色身份推演对话和剧情
                </p>
              </div>
              <div className="flex items-center gap-2 pt-1">
                <Button onClick={handleSave} size="sm" data-icon="inline-start">
                  <Check />
                  保存
                </Button>
                <Button variant="ghost" size="sm" onClick={handleCancel} data-icon="inline-start">
                  <X />
                  取消
                </Button>
              </div>
            </>
          ) : (
            <>
              {character.personality && (
                <div>
                  <p className="mb-1 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    性格特点
                  </p>
                  <p className="text-sm text-card-foreground/90 whitespace-pre-wrap leading-relaxed">
                    {character.personality}
                  </p>
                </div>
              )}
              {character.background && (
                <div>
                  <p className="mb-1 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    背景故事
                  </p>
                  <p className="text-sm text-card-foreground/90 whitespace-pre-wrap leading-relaxed">
                    {character.background}
                  </p>
                </div>
              )}
              {character.model_name && (
                <div>
                  <p className="mb-1 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    AI 模型
                  </p>
                  <p className="text-sm text-card-foreground/90 flex items-center gap-1.5">
                    <Cpu className="size-3.5 text-muted-foreground" />
                    {character.model_name}
                  </p>
                </div>
              )}
              {charFactions.length > 0 && (
                <div>
                  <p className="mb-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
                    <Swords className="size-3" />
                    势力归属
                  </p>
                  <div className="flex flex-wrap gap-1.5">
                    {charFactions.map((f) => (
                      <Badge key={f.id} variant="secondary" className="text-xs">
                        {f.faction_name}
                        {f.role && f.role !== "成员" && ` · ${f.role}`}
                        <span className="ml-1 text-muted-foreground/50 text-[10px]">
                          {formatChapterRange(f.start_chapter_title, f.end_chapter_title)}
                        </span>
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
              {charRelations.length > 0 && (
                <div>
                  <p className="mb-1.5 text-xs font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
                    <Users className="size-3" />
                    角色关系
                  </p>
                  <div className="flex flex-col gap-1">
                    {charRelations.map((r) => {
                      const otherName = r.char_a_id === character.id ? r.char_b_name : r.char_a_name;
                      const range = formatChapterRange(r.start_chapter_title, r.end_chapter_title);
                      return (
                        <div key={r.id} className="flex items-center gap-2 text-xs text-muted-foreground">
                          <span className="text-foreground/80 font-medium">{otherName}</span>
                          <span className="text-muted-foreground/50">—</span>
                          <span>{r.relationship_type || r.description || "未标注"}</span>
                          {range && (
                            <span className="text-muted-foreground/40 text-[10px]">
                              ({range})
                            </span>
                          )}
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
              <Button
                variant="ghost"
                size="xs"
                onClick={onDelete}
                className="text-destructive/80 hover:text-destructive self-start"
                data-icon="inline-start"
              >
                <Trash2 />
                删除角色
              </Button>
            </>
          )}
        </CardContent>
      )}
    </Card>
  );
};

const CharactersPage = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const [characters, setCharacters] = useState<Character[]>([]);
  const [models, setModels] = useState<AiConfig[]>([]);
  const [relations, setRelations] = useState<CharacterRelation[]>([]);
  const [factions, setFactions] = useState<CharacterFaction[]>([]);
  const [raceEntries, setRaceEntries] = useState<WorldviewEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [newModelId, setNewModelId] = useState("");
  const [search, setSearch] = useState("");

  const loadData = async () => {
    if (!projectId) return;
    try {
      const [charData, modelData, relData, facData, worldviewData] = await Promise.all([
        characterApi.list(projectId),
        aiApi.listModels(),
        relationApi.listRelations(projectId),
        relationApi.listFactions(projectId),
        worldviewApi.list(projectId),
      ]);
      setCharacters(charData);
      setModels(modelData);
      setRelations(relData);
      setFactions(facData);
      setRaceEntries(worldviewData.filter((e) => e.category === "race"));
    } catch (err) {
      console.error("Failed to load characters:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadData();
  }, [projectId]);

  const handleCreate = async () => {
    if (!projectId || !newName.trim()) return;
    try {
      const character = await characterApi.create(
        projectId,
        newName.trim(),
        "",
        "",
        "",
        "",
        newModelId || null,
      );
      setCharacters((prev) => [...prev, character]);
      setNewName("");
      setNewModelId("");
      setShowCreate(false);
    } catch (err) {
      console.error("Failed to create character:", err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await characterApi.delete(id);
      setCharacters((prev) => prev.filter((c) => c.id !== id));
    } catch (err) {
      console.error("Failed to delete character:", err);
    }
  };

  const handleUpdate = async (
    id: string,
    data: {
      name: string;
      description: string;
      personality: string;
      background: string;
      race: string;
      modelId?: string | null;
    },
  ) => {
    try {
      const updated = await characterApi.update(
        id,
        data.name,
        data.description,
        data.personality,
        data.background,
        data.race,
        data.modelId,
      );
      setCharacters((prev) =>
        prev.map((c) => (c.id === id ? updated : c)),
      );
    } catch (err) {
      console.error("Failed to update character:", err);
    }
  };

  const filteredCharacters = search.trim()
    ? characters.filter((c) =>
        c.name.toLowerCase().includes(search.toLowerCase()) ||
        c.description.toLowerCase().includes(search.toLowerCase()) ||
        c.race.toLowerCase().includes(search.toLowerCase()),
      )
    : characters;

  const [viewMode, setViewMode] = useState<"list" | "graph">("list");

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between px-5 h-11 shrink-0">
        <div className="flex items-center gap-2">
          <h1 className="text-sm font-medium text-foreground">角色管理</h1>
          <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as "list" | "graph")}>
            <TabsList variant="line" className="h-7">
              <TabsTrigger value="list" data-icon="inline-start">
                <UserRound className="size-3" />
                列表
              </TabsTrigger>
              <TabsTrigger value="graph" data-icon="inline-start">
                <Network className="size-3" />
                关系图
              </TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
        {viewMode === "list" && (
          <Button onClick={() => setShowCreate(true)} data-icon="inline-start">
            <Plus />
            新建角色
          </Button>
        )}
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-5">
        {viewMode === "graph" ? (
          <CharacterGraph
            projectId={projectId!}
            characters={characters}
            relations={relations}
            factions={factions}
          />
        ) : loading ? (
          <div className="flex h-32 items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <Spinner className="size-5 text-primary" />
              <p className="text-sm text-muted-foreground">加载中...</p>
            </div>
          </div>
        ) : characters.length === 0 ? (
          <Empty className="h-[calc(100vh-240px)] border-transparent">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <UserRound />
              </EmptyMedia>
              <EmptyTitle>还没有角色</EmptyTitle>
              <EmptyDescription>创建你的第一个角色</EmptyDescription>
              <EmptyContent>
                <Button onClick={() => setShowCreate(true)} data-icon="inline-start">
                  <Plus />
                  新建角色
                </Button>
              </EmptyContent>
            </EmptyHeader>
          </Empty>
        ) : (
          <>
            <div className="relative mb-4 max-w-xs">
              <Search className="absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
              <Input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="搜索角色..."
                className="pl-9"
              />
            </div>
            <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
              {filteredCharacters.map((character) => (
                <CharacterCard
                  key={character.id}
                  character={character}
                  relations={relations}
                  factions={factions}
                  models={models}
                  raceEntries={raceEntries}
                  onDelete={() => handleDelete(character.id)}
                  onUpdate={(data) => handleUpdate(character.id, data)}
                />
              ))}
            </div>
            {filteredCharacters.length === 0 && search.trim() && (
              <Empty className="py-12 border-transparent">
                <EmptyHeader>
                  <EmptyDescription>未找到匹配「{search}」的角色</EmptyDescription>
                </EmptyHeader>
              </Empty>
            )}
          </>
        )}
      </div>

      <Dialog open={showCreate} onOpenChange={(isOpen) => { if (!isOpen) setShowCreate(false); }}>
        <DialogContent className="sm:max-w-sm">
          <DialogHeader>
            <DialogTitle>新建角色</DialogTitle>
            <DialogDescription>为你的故事添加一个新角色</DialogDescription>
          </DialogHeader>
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1.5">
              <Label className="text-sm font-medium text-foreground">角色姓名</Label>
              <Input
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder="角色姓名"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleCreate();
                }}
              />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label className="text-sm font-medium text-foreground">种族</Label>
              <Input
                value=""
                onChange={() => {}}
                placeholder="选择或输入种族"
                list="race-list-create"
              />
              <datalist id="race-list-create">
                {raceEntries.map((e) => (
                  <option key={e.id} value={e.title} />
                ))}
              </datalist>
            </div>
            <div className="flex flex-col gap-1.5">
              <Label className="text-sm font-medium text-foreground">AI 模型</Label>
              <Select value={newModelId} onValueChange={(v) => setNewModelId(v ?? "")}>
                <SelectTrigger>
                  <SelectValue>
                    {newModelId ? (models.find((m) => m.id === newModelId)?.name ?? newModelId) : "默认模型"}
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectItem value="">
                      默认模型
                    </SelectItem>
                    {models.map((m) => (
                      <SelectItem key={m.id} value={m.id}>
                        {m.name}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground/60">
                可选。指定后 AI 将以该角色身份参与推演
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreate(false)}>
              取消
            </Button>
            <Button onClick={handleCreate} disabled={!newName.trim()}>
              创建
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default CharactersPage;
