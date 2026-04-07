import { useState } from "react";
import { useParams } from "react-router-dom";
import {
  Plus,
  Globe,
  Trash2,
  Pencil,
  X,
  Check,
  ChevronDown,
  MapPin,
  Clock,
  Palette,
  Wand2,
  Cpu,
  Bookmark,
  Swords,
  Users,
  Gem,
} from "lucide-react";
import { worldviewApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { WorldviewEntry } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Spinner } from "@/components/ui/spinner";
import { Card, CardContent } from "@/components/ui/card";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from "@/components/ui/empty";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { Select, SelectTrigger, SelectContent, SelectGroup, SelectItem, SelectValue } from "@/components/ui/select";
import { useResource } from "@/hooks/useResource";
import { useDialog } from "@/hooks/useDialog";

const categories = [
  { key: "all", label: "全部", icon: Globe },
  { key: "geography", label: "地理", icon: MapPin },
  { key: "faction", label: "势力", icon: Swords },
  { key: "race", label: "种族", icon: Users },
  { key: "history", label: "历史", icon: Clock },
  { key: "culture", label: "文化", icon: Palette },
  { key: "magic", label: "魔法体系", icon: Wand2 },
  { key: "technology", label: "科技", icon: Cpu },
  { key: "item", label: "物品", icon: Gem },
  { key: "other", label: "其他", icon: Bookmark },
];

const categoryMap: Record<string, string> = {
  geography: "地理",
  faction: "势力",
  race: "种族",
  history: "历史",
  culture: "文化",
  magic: "魔法体系",
  technology: "科技",
  item: "物品",
  other: "其他",
};

const categoryVariant: Record<string, "default" | "secondary" | "outline" | "ghost"> = {
  geography: "secondary",
  faction: "default",
  race: "secondary",
  history: "secondary",
  culture: "secondary",
  magic: "default",
  technology: "secondary",
  item: "secondary",
  other: "ghost",
};

const EntryCard = ({
  entry,
  onDelete,
  onUpdate,
}: {
  entry: WorldviewEntry;
  onDelete: () => void;
  onUpdate: (data: {
    category: string;
    title: string;
    content: string;
  }) => void;
}) => {
  const [editing, setEditing] = useState(false);
  const [category, setCategory] = useState(entry.category);
  const [title, setTitle] = useState(entry.title);
  const [content, setContent] = useState(entry.content);
  const [expanded, setExpanded] = useState(false);

  const handleSave = async () => {
    await onUpdate({ category, title, content });
    setEditing(false);
  };

  const handleCancel = () => {
    setCategory(entry.category);
    setTitle(entry.title);
    setContent(entry.content);
    setEditing(false);
  };

  return (
    <Card className="transition-all duration-200 hover:ring-primary/20">
      <div
        className="flex items-center gap-3.5 cursor-pointer select-none"
        onClick={() => !editing && setExpanded(!expanded)}
      >
        <Badge variant={categoryVariant[entry.category] || "ghost"}>
          {categoryMap[entry.category] || entry.category}
        </Badge>
        <p className="flex-1 font-semibold text-card-foreground truncate">{entry.title}</p>
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
                <Select value={category} onValueChange={(v) => { if (v) setCategory(v); }}>
                  <SelectTrigger size="sm" className="w-auto">
                  <SelectValue>{categoryMap[category] || category}</SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {categories
                        .filter((c) => c.key !== "all")
                        .map((c) => (
                          <SelectItem key={c.key} value={c.key}>
                            {c.label}
                          </SelectItem>
                        ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
                <Input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="词条标题"
                />
              </div>
              <Textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                rows={5}
                placeholder="详细描述..."
              />
              <div className="flex items-center gap-2">
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
              <p className="text-sm text-card-foreground/90 whitespace-pre-wrap leading-relaxed">
                {entry.content || "暂无内容"}
              </p>
              <Button
                variant="ghost"
                size="xs"
                onClick={onDelete}
                className="text-destructive/80 hover:text-destructive self-start"
                data-icon="inline-start"
              >
                <Trash2 />
                删除词条
              </Button>
            </>
          )}
        </CardContent>
      )}
    </Card>
  );
};

const WorldviewPage = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const { items: entries, loading, append, replace, remove } = useResource(
    () => worldviewApi.list(projectId!),
    [projectId],
  );
  const dialog = useDialog();
  const [activeCategory, setActiveCategory] = useState("all");
  const [newCategory, setNewCategory] = useState("geography");
  const [newTitle, setNewTitle] = useState("");
  const [newContent, setNewContent] = useState("");

  const handleCreate = async () => {
    if (!projectId || !newTitle.trim()) return;
    try {
      const entry = await worldviewApi.create(projectId, newCategory, newTitle.trim(), newContent.trim());
      append(entry);
      setNewTitle("");
      setNewContent("");
      dialog.close();
    } catch (err) {
      console.error("Failed to create entry:", err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await worldviewApi.delete(id);
      remove(id);
    } catch (err) {
      console.error("Failed to delete entry:", err);
    }
  };

  const handleUpdate = async (
    id: string,
    data: { category: string; title: string; content: string },
  ) => {
    try {
      const updated = await worldviewApi.update(id, data.category, data.title, data.content);
      replace(id, updated);
    } catch (err) {
      console.error("Failed to update entry:", err);
    }
  };

  const filteredEntries =
    activeCategory === "all"
      ? entries
      : entries.filter((e) => e.category === activeCategory);

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between px-5 h-11 shrink-0">
        <h1 className="text-sm font-medium text-foreground">世界观</h1>
        <Button onClick={() => dialog.show()} data-icon="inline-start">
          <Plus />
          新建词条
        </Button>
      </div>

      <div className="border-b border-border px-6 py-2.5">
        <Tabs
          value={activeCategory}
          onValueChange={setActiveCategory}
        >
          <TabsList variant="line">
            {categories.map((cat) => (
              <TabsTrigger key={cat.key} value={cat.key} data-icon="inline-start">
                <cat.icon />
                {cat.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-5">
        {loading ? (
          <div className="flex h-32 items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <Spinner className="size-5 text-primary" />
              <p className="text-sm text-muted-foreground">加载中...</p>
            </div>
          </div>
        ) : filteredEntries.length === 0 ? (
          <Empty className="h-[calc(100vh-280px)] border-transparent">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Globe />
              </EmptyMedia>
              <EmptyTitle>
                {activeCategory === "all" ? "还没有世界观词条" : "该分类下暂无词条"}
              </EmptyTitle>
              <EmptyDescription>
                {activeCategory === "all" ? "开始构建你的故事世界" : "试试其他分类"}
              </EmptyDescription>
              {activeCategory === "all" && (
                <EmptyContent>
                  <Button onClick={() => dialog.show()} data-icon="inline-start">
                    <Plus />
                    新建词条
                  </Button>
                </EmptyContent>
              )}
            </EmptyHeader>
          </Empty>
        ) : (
          <div className="flex flex-col gap-3">
            {filteredEntries.map((entry) => (
              <EntryCard
                key={entry.id}
                entry={entry}
                onDelete={() => handleDelete(entry.id)}
                onUpdate={(data) => handleUpdate(entry.id, data)}
              />
            ))}
          </div>
        )}
      </div>

      <Dialog open={dialog.open} onOpenChange={dialog.onOpenChange}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>新建世界观词条</DialogTitle>
            <DialogDescription>定义故事世界中的一个元素</DialogDescription>
          </DialogHeader>
          <div className="flex flex-col gap-4">
            <div className="flex flex-col gap-1.5">
              <Label>分类</Label>
              <Select value={newCategory} onValueChange={(v) => { if (v) setNewCategory(v); }}>
                <SelectTrigger>
                  <SelectValue>{categoryMap[newCategory] || newCategory}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {categories
                      .filter((c) => c.key !== "all")
                      .map((c) => (
                        <SelectItem key={c.key} value={c.key}>
                          {c.label}
                        </SelectItem>
                      ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
            <div className="flex flex-col gap-1.5">
              <Label>词条标题</Label>
              <Input
                value={newTitle}
                onChange={(e) => setNewTitle(e.target.value)}
                placeholder="词条标题"
                autoFocus
              />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label>详细描述</Label>
              <Textarea
                value={newContent}
                onChange={(e) => setNewContent(e.target.value)}
                placeholder="详细描述..."
                rows={4}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={dialog.close}>
              取消
            </Button>
            <Button onClick={handleCreate} disabled={!newTitle.trim()}>
              创建
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default WorldviewPage;
