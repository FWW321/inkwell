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
} from "lucide-react";
import { characterApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { Character } from "@/lib/types";

const CharacterCard = ({
  character,
  onDelete,
  onUpdate,
}: {
  character: Character;
  onDelete: () => void;
  onUpdate: (data: {
    name: string;
    description: string;
    personality: string;
    background: string;
  }) => void;
}) => {
  const [editing, setEditing] = useState(false);
  const [name, setName] = useState(character.name);
  const [description, setDescription] = useState(character.description);
  const [personality, setPersonality] = useState(character.personality);
  const [background, setBackground] = useState(character.background);
  const [expanded, setExpanded] = useState(false);

  const handleSave = async () => {
    await onUpdate({ name, description, personality, background });
    setEditing(false);
  };

  const handleCancel = () => {
    setName(character.name);
    setDescription(character.description);
    setPersonality(character.personality);
    setBackground(character.background);
    setEditing(false);
  };

  return (
    <div className={cn(
      "group rounded-xl border border-border bg-card transition-all duration-200",
      "hover:border-primary/20 hover:bg-card/80",
    )}>
      <div
        className="flex items-center gap-3.5 px-5 py-4 cursor-pointer select-none"
        onClick={() => !editing && setExpanded(!expanded)}
      >
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-xl bg-primary/12">
          <UserRound className="h-5 w-5 text-primary/80" />
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-foreground truncate">
            {character.name}
          </h3>
          {character.description && (
            <p className="mt-0.5 text-sm text-muted-foreground truncate">
              {character.description}
            </p>
          )}
        </div>
        <div className="flex items-center gap-1">
          {expanded && !editing && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                setEditing(true);
              }}
              className="flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground opacity-0 transition-all group-hover:opacity-100 hover:bg-secondary hover:text-foreground"
            >
              <Pencil className="h-3.5 w-3.5" />
            </button>
          )}
          {!editing && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                setExpanded(!expanded);
              }}
              className={cn(
                "flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-all",
                "hover:bg-secondary hover:text-foreground",
                expanded && "rotate-180",
              )}
            >
              <ChevronDown className="h-4 w-4" />
            </button>
          )}
        </div>
      </div>

      {expanded && (
        <div className="border-t border-border px-5 py-4 space-y-4 animate-fade-in">
          {editing ? (
            <>
              <div>
                <label className="mb-1.5 block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                  姓名
                </label>
                <input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
                />
              </div>
              <div>
                <label className="mb-1.5 block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                  外貌描述
                </label>
                <textarea
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  rows={2}
                  className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
                />
              </div>
              <div>
                <label className="mb-1.5 block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                  性格特点
                </label>
                <textarea
                  value={personality}
                  onChange={(e) => setPersonality(e.target.value)}
                  rows={2}
                  className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
                />
              </div>
              <div>
                <label className="mb-1.5 block text-xs font-medium text-muted-foreground uppercase tracking-wider">
                  背景故事
                </label>
                <textarea
                  value={background}
                  onChange={(e) => setBackground(e.target.value)}
                  rows={3}
                  className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
                />
              </div>
              <div className="flex items-center gap-2 pt-1">
                <button
                  onClick={handleSave}
                  className={cn(
                    "flex items-center gap-1.5 rounded-lg px-4 py-2 text-sm font-medium text-primary-foreground transition-all",
                    "bg-primary hover:bg-primary/90 active:scale-[0.97]",
                  )}
                >
                  <Check className="h-3.5 w-3.5" />
                  保存
                </button>
                <button
                  onClick={handleCancel}
                  className="flex items-center gap-1.5 rounded-lg px-4 py-2 text-sm text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
                >
                  <X className="h-3.5 w-3.5" />
                  取消
                </button>
              </div>
            </>
          ) : (
            <>
              {character.personality && (
                <div>
                  <p className="mb-1 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    性格特点
                  </p>
                  <p className="text-sm text-foreground/90 whitespace-pre-wrap leading-relaxed">
                    {character.personality}
                  </p>
                </div>
              )}
              {character.background && (
                <div>
                  <p className="mb-1 text-xs font-medium text-muted-foreground uppercase tracking-wider">
                    背景故事
                  </p>
                  <p className="text-sm text-foreground/90 whitespace-pre-wrap leading-relaxed">
                    {character.background}
                  </p>
                </div>
              )}
              <button
                onClick={onDelete}
                className="flex items-center gap-1.5 text-xs text-destructive/80 transition-colors hover:text-destructive hover:underline"
              >
                <Trash2 className="h-3 w-3" />
                删除角色
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
};

const CharactersPage = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const [characters, setCharacters] = useState<Character[]>([]);
  const [loading, setLoading] = useState(true);
  const [showCreate, setShowCreate] = useState(false);
  const [newName, setNewName] = useState("");
  const [search, setSearch] = useState("");

  const loadCharacters = async () => {
    if (!projectId) return;
    try {
      const data = await characterApi.list(projectId);
      setCharacters(data);
    } catch (err) {
      console.error("Failed to load characters:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadCharacters();
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
      );
      setCharacters((prev) => [...prev, character]);
      setNewName("");
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
    },
  ) => {
    try {
      const updated = await characterApi.update(
        id,
        data.name,
        data.description,
        data.personality,
        data.background,
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
        c.description.toLowerCase().includes(search.toLowerCase()),
      )
    : characters;

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-lg font-semibold text-foreground">角色管理</h1>
          <p className="mt-0.5 text-sm text-muted-foreground">
            创建和管理故事中的角色
          </p>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className={cn(
            "flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-medium text-primary-foreground transition-all",
            "bg-primary hover:bg-primary/90 active:scale-[0.97]",
          )}
        >
          <Plus className="h-4 w-4" />
          新建角色
        </button>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-5">
        {loading ? (
          <div className="flex h-32 items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
              <p className="text-sm text-muted-foreground">加载中...</p>
            </div>
          </div>
        ) : characters.length === 0 ? (
          <div className="flex h-[calc(100vh-240px)] flex-col items-center justify-center gap-5">
            <div className="flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10">
              <UserRound className="h-10 w-10 text-primary/60" />
            </div>
            <div className="text-center">
              <p className="text-lg font-semibold text-foreground">还没有角色</p>
              <p className="mt-1 text-sm text-muted-foreground">创建你的第一个角色</p>
            </div>
            <button
              onClick={() => setShowCreate(true)}
              className={cn(
                "flex items-center gap-2 rounded-lg px-5 py-2.5 text-sm font-medium text-primary-foreground transition-all",
                "bg-primary hover:bg-primary/90 active:scale-[0.97]",
              )}
            >
              <Plus className="h-4 w-4" />
              新建角色
            </button>
          </div>
        ) : (
          <>
            <div className="relative mb-4 max-w-xs">
              <Search className="absolute left-3 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-muted-foreground/60" />
              <input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="搜索角色..."
                className="w-full rounded-lg border border-input bg-background py-2 pl-9 pr-3 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
              />
            </div>
            <div className="grid grid-cols-1 gap-3 lg:grid-cols-2">
              {filteredCharacters.map((character) => (
                <CharacterCard
                  key={character.id}
                  character={character}
                  onDelete={() => handleDelete(character.id)}
                  onUpdate={(data) => handleUpdate(character.id, data)}
                />
              ))}
            </div>
            {filteredCharacters.length === 0 && search.trim() && (
              <div className="py-12 text-center text-sm text-muted-foreground">
                未找到匹配「{search}」的角色
              </div>
            )}
          </>
        )}
      </div>

      {showCreate && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          onClick={() => setShowCreate(false)}
        >
          <div
            className="animate-fade-in w-full max-w-sm rounded-xl border border-border bg-card p-6 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <h2 className="mb-1 text-lg font-semibold text-foreground">新建角色</h2>
            <p className="mb-5 text-sm text-muted-foreground">为你的故事添加一个新角色</p>
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="角色姓名"
              className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreate();
              }}
            />
            <div className="mt-5 flex justify-end gap-2">
              <button
                onClick={() => setShowCreate(false)}
                className="rounded-lg px-4 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-secondary hover:text-secondary-foreground"
              >
                取消
              </button>
              <button
                onClick={handleCreate}
                disabled={!newName.trim()}
                className={cn(
                  "rounded-lg px-5 py-2.5 text-sm font-medium text-primary-foreground transition-all",
                  "bg-primary hover:bg-primary/90 active:scale-[0.97] disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100",
                )}
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default CharactersPage;
