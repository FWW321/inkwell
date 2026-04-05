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
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Card, CardContent } from "@/components/ui/card";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from "@/components/ui/empty";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";

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
          {character.description && (
            <p className="mt-0.5 text-sm text-muted-foreground truncate">
              {character.description}
            </p>
          )}
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
              <div className="flex flex-col gap-1.5">
                <Label className="text-xs uppercase tracking-wider">姓名</Label>
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                />
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
        <Button onClick={() => setShowCreate(true)} data-icon="inline-start">
          <Plus />
          新建角色
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-5">
        {loading ? (
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
