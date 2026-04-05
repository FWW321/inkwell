import { useState, useEffect } from "react";
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
} from "lucide-react";
import { worldviewApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { WorldviewEntry } from "@/lib/types";

const categories = [
  { key: "all", label: "全部", icon: Globe },
  { key: "geography", label: "地理", icon: MapPin },
  { key: "history", label: "历史", icon: Clock },
  { key: "culture", label: "文化", icon: Palette },
  { key: "magic", label: "魔法体系", icon: Wand2 },
  { key: "technology", label: "科技", icon: Cpu },
  { key: "other", label: "其他", icon: Bookmark },
];

const categoryMap: Record<string, string> = {
  geography: "地理",
  history: "历史",
  culture: "文化",
  magic: "魔法体系",
  technology: "科技",
  other: "其他",
};

const categoryColors: Record<string, string> = {
  geography: "bg-blue-500/12 text-blue-400",
  history: "bg-amber-500/12 text-amber-400",
  culture: "bg-pink-500/12 text-pink-400",
  magic: "bg-violet-500/12 text-violet-400",
  technology: "bg-cyan-500/12 text-cyan-400",
  other: "bg-muted/80 text-muted-foreground",
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
    <div className={cn(
      "group rounded-xl border border-border bg-card transition-all duration-200",
      "hover:border-primary/20 hover:bg-card/80",
    )}>
      <div
        className="flex items-center gap-3.5 px-5 py-4 cursor-pointer select-none"
        onClick={() => !editing && setExpanded(!expanded)}
      >
        <span className={cn(
          "shrink-0 rounded-lg px-2.5 py-1 text-xs font-medium",
          categoryColors[entry.category] || categoryColors.other,
        )}>
          {categoryMap[entry.category] || entry.category}
        </span>
        <h3 className="flex-1 font-semibold text-foreground truncate">{entry.title}</h3>
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
              <div className="flex gap-2">
                <select
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
                  className="rounded-lg border border-input bg-background px-2.5 py-2 text-sm outline-none cursor-pointer transition-colors"
                  style={{ colorScheme: "dark" }}
                >
                  {categories
                    .filter((c) => c.key !== "all")
                    .map((c) => (
                      <option key={c.key} value={c.key}>
                        {c.label}
                      </option>
                    ))}
                </select>
                <input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  className="flex-1 rounded-lg border border-input bg-background px-3 py-2 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
                  placeholder="词条标题"
                />
              </div>
              <textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                rows={5}
                className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
                placeholder="详细描述..."
              />
              <div className="flex items-center gap-2">
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
              <p className="text-sm text-foreground/90 whitespace-pre-wrap leading-relaxed">
                {entry.content || "暂无内容"}
              </p>
              <button
                onClick={onDelete}
                className="flex items-center gap-1.5 text-xs text-destructive/80 transition-colors hover:text-destructive hover:underline"
              >
                <Trash2 className="h-3 w-3" />
                删除词条
              </button>
            </>
          )}
        </div>
      )}
    </div>
  );
};

const WorldviewPage = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const [entries, setEntries] = useState<WorldviewEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeCategory, setActiveCategory] = useState("all");
  const [showCreate, setShowCreate] = useState(false);
  const [newCategory, setNewCategory] = useState("geography");
  const [newTitle, setNewTitle] = useState("");
  const [newContent, setNewContent] = useState("");

  const loadEntries = async () => {
    if (!projectId) return;
    try {
      const data = await worldviewApi.list(projectId);
      setEntries(data);
    } catch (err) {
      console.error("Failed to load worldview:", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadEntries();
  }, [projectId]);

  const handleCreate = async () => {
    if (!projectId || !newTitle.trim()) return;
    try {
      const entry = await worldviewApi.create(
        projectId,
        newCategory,
        newTitle.trim(),
        newContent.trim(),
      );
      setEntries((prev) => [...prev, entry]);
      setNewTitle("");
      setNewContent("");
      setShowCreate(false);
    } catch (err) {
      console.error("Failed to create entry:", err);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await worldviewApi.delete(id);
      setEntries((prev) => prev.filter((e) => e.id !== id));
    } catch (err) {
      console.error("Failed to delete entry:", err);
    }
  };

  const handleUpdate = async (
    id: string,
    data: { category: string; title: string; content: string },
  ) => {
    try {
      const updated = await worldviewApi.update(
        id,
        data.category,
        data.title,
        data.content,
      );
      setEntries((prev) =>
        prev.map((e) => (e.id === id ? updated : e)),
      );
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
      <div className="flex items-center justify-between border-b border-border px-6 py-4">
        <div>
          <h1 className="text-lg font-semibold text-foreground">世界观</h1>
          <p className="mt-0.5 text-sm text-muted-foreground">
            构建故事世界的地理、历史、文化和规则
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
          新建词条
        </button>
      </div>

      <div className="flex gap-1 overflow-x-auto border-b border-border px-6 py-2.5 scrollbar-none">
        {categories.map((cat) => (
          <button
            key={cat.key}
            onClick={() => setActiveCategory(cat.key)}
            className={cn(
              "flex shrink-0 items-center gap-1.5 rounded-lg px-3 py-1.5 text-sm font-medium transition-all",
              activeCategory === cat.key
                ? "bg-primary/15 text-primary"
                : "text-muted-foreground hover:bg-secondary hover:text-foreground active:scale-[0.97]",
            )}
          >
            <cat.icon className="h-3.5 w-3.5" />
            {cat.label}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-5">
        {loading ? (
          <div className="flex h-32 items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
              <p className="text-sm text-muted-foreground">加载中...</p>
            </div>
          </div>
        ) : filteredEntries.length === 0 ? (
          <div className="flex h-[calc(100vh-280px)] flex-col items-center justify-center gap-5">
            <div className="flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10">
              <Globe className="h-10 w-10 text-primary/60" />
            </div>
            <div className="text-center">
              <p className="text-lg font-semibold text-foreground">
                {activeCategory === "all" ? "还没有世界观词条" : "该分类下暂无词条"}
              </p>
              <p className="mt-1 text-sm text-muted-foreground">
                {activeCategory === "all" ? "开始构建你的故事世界" : "试试其他分类"}
              </p>
            </div>
            {activeCategory === "all" && (
              <button
                onClick={() => setShowCreate(true)}
                className={cn(
                  "flex items-center gap-2 rounded-lg px-5 py-2.5 text-sm font-medium text-primary-foreground transition-all",
                  "bg-primary hover:bg-primary/90 active:scale-[0.97]",
                )}
              >
                <Plus className="h-4 w-4" />
                新建词条
              </button>
            )}
          </div>
        ) : (
          <div className="space-y-3">
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

      {showCreate && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          onClick={() => setShowCreate(false)}
        >
          <div
            className="animate-fade-in w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <h2 className="mb-1 text-lg font-semibold text-foreground">新建世界观词条</h2>
            <p className="mb-5 text-sm text-muted-foreground">定义故事世界中的一个元素</p>
            <div className="space-y-3">
              <select
                value={newCategory}
                onChange={(e) => setNewCategory(e.target.value)}
                className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none cursor-pointer transition-colors"
                style={{ colorScheme: "dark" }}
              >
                {categories
                  .filter((c) => c.key !== "all")
                  .map((c) => (
                    <option key={c.key} value={c.key}>
                      {c.label}
                    </option>
                  ))}
              </select>
              <input
                value={newTitle}
                onChange={(e) => setNewTitle(e.target.value)}
                placeholder="词条标题"
                className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary transition-colors"
                autoFocus
              />
              <textarea
                value={newContent}
                onChange={(e) => setNewContent(e.target.value)}
                placeholder="详细描述..."
                rows={4}
                className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/50 focus:border-primary resize-none transition-colors"
              />
            </div>
            <div className="mt-5 flex justify-end gap-2">
              <button
                onClick={() => setShowCreate(false)}
                className="rounded-lg px-4 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-secondary hover:text-secondary-foreground"
              >
                取消
              </button>
              <button
                onClick={handleCreate}
                disabled={!newTitle.trim()}
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

export default WorldviewPage;
