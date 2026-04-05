import { useState, useEffect, useMemo } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  ChevronRight,
  Plus,
  FileText,
  FolderOpen,
  MoreHorizontal,
  Trash2,
  Pencil,
  Search,
  BookOpen,
} from "lucide-react";
import { outlineApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { OutlineNode } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible";

interface ChapterTreeProps {
  projectId: string;
}

const ChapterTree = ({ projectId }: ChapterTreeProps) => {
  const navigate = useNavigate();
  const { chapterId } = useParams();
  const [nodes, setNodes] = useState<OutlineNode[]>([]);
  const [childrenMap, setChildrenMap] = useState<Record<string, OutlineNode[]>>({});
  const [expandedIds, setExpandedIds] = useState<Set<string>>(new Set());
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState("");
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");

  useEffect(() => {
    const loadData = async () => {
      try {
        const topNodes = await outlineApi.list(projectId);
        setNodes(topNodes);
        const childMap: Record<string, OutlineNode[]> = {};
        for (const node of topNodes) {
          if (node.node_type === "volume") {
            childMap[node.id] = await outlineApi.list(projectId, node.id);
          }
        }
        setChildrenMap(childMap);
      } catch (err) {
        console.error("Failed to load chapter tree:", err);
      } finally {
        setLoading(false);
      }
    };
    loadData();
  }, [projectId]);

  const filteredNodes = useMemo(() => {
    if (!search.trim()) return nodes;
    const q = search.toLowerCase();
    return nodes.filter((n) =>
      n.title.toLowerCase().includes(q) ||
      (childrenMap[n.id] ?? []).some((c) => c.title.toLowerCase().includes(q)),
    );
  }, [nodes, search, childrenMap]);

  const toggleExpand = (id: string) => {
    setExpandedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const handleAddVolume = async () => {
    try {
      const volumeCount = nodes.filter((n) => n.node_type === "volume").length;
      const node = await outlineApi.create(projectId, null, "volume", `第${volumeCount + 1}卷`);
      setNodes((prev) => [...prev, node]);
      setChildrenMap((prev) => ({ ...prev, [node.id]: [] }));
    } catch (err) {
      console.error("Failed to create volume:", err);
    }
  };

  const handleAddChapter = async (parentId: string | null) => {
    try {
      const siblings = parentId
        ? childrenMap[parentId] || []
        : nodes;
      const chapterCount = siblings.filter((n) => n.node_type === "chapter").length;
      const chap = await outlineApi.create(
        projectId,
        parentId,
        "chapter",
        `第${chapterCount + 1}章`,
      );
      if (parentId) {
        setChildrenMap((prev) => ({
          ...prev,
          [parentId]: [...(prev[parentId] || []), chap],
        }));
        if (!expandedIds.has(parentId)) {
          setExpandedIds((prev) => new Set(prev).add(parentId));
        }
      } else {
        setNodes((prev) => [...prev, chap]);
      }
      navigate(`/project/${projectId}/write/${chap.id}`);
    } catch (err) {
      console.error("Failed to create chapter:", err);
    }
  };

  const handleDelete = async (id: string, parentId: string | null) => {
    try {
      await outlineApi.delete(id);
      if (parentId) {
        setChildrenMap((prev) => ({
          ...prev,
          [parentId]: (prev[parentId] || []).filter((n) => n.id !== id),
        }));
      } else {
        setNodes((prev) => prev.filter((n) => n.id !== id));
        setChildrenMap((prev) => {
          const next = { ...prev };
          delete next[id];
          return next;
        });
      }
      if (chapterId === id) {
        navigate(`/project/${projectId}/write`);
      }
    } catch (err) {
      console.error("Failed to delete node:", err);
    }
  };

  const finishEditing = async (id: string, parentId: string | null) => {
    if (!editingTitle.trim()) {
      setEditingId(null);
      return;
    }
    try {
      await outlineApi.rename(id, editingTitle.trim());
      const updateList = (list: OutlineNode[]) =>
        list.map((n) => (n.id === id ? { ...n, title: editingTitle.trim() } : n));
      if (parentId) {
        setChildrenMap((prev) => ({
          ...prev,
          [parentId]: updateList(prev[parentId] || []),
        }));
      } else {
        setNodes(updateList);
      }
    } catch (err) {
      console.error("Failed to rename node:", err);
    }
    setEditingId(null);
  };

  if (loading) {
    return (
      <div className="p-3 flex flex-col gap-2">
        <Skeleton className="h-4 w-16" />
        <Skeleton className="h-3 w-24" />
        <Skeleton className="h-3 w-20" />
      </div>
    );
  }

  const renderNode = (node: OutlineNode, parentId: string | null, depth: number = 0) => {
    const isVolume = node.node_type === "volume";
    const expanded = expandedIds.has(node.id);
    const children = (childrenMap[node.id] || []).filter(
      (c) => !search.trim() || c.title.toLowerCase().includes(search.toLowerCase()),
    );
    const isActive = chapterId === node.id;

    return (
      <Collapsible key={node.id} open={isVolume ? expanded : false} onOpenChange={() => isVolume && toggleExpand(node.id)}>
        <div
          className={cn(
            "group flex items-center gap-1.5 rounded-md px-2 py-1.5 text-[13px] transition-colors cursor-pointer",
            isActive
              ? "bg-primary/12 text-primary font-medium"
              : "text-sidebar-foreground/80 hover:bg-secondary/60 hover:text-sidebar-foreground",
          )}
          style={{ paddingLeft: `${depth * 14 + 8}px` }}
          onClick={() => {
            if (isVolume) {
              toggleExpand(node.id);
            } else {
              navigate(`/project/${projectId}/write/${node.id}`);
            }
          }}
        >
          {isVolume ? (
            <ChevronRight
              className={cn(
                "size-3.5 shrink-0 text-muted-foreground/60 transition-transform duration-150",
                expanded && "rotate-90",
              )}
            />
          ) : (
            <FileText className="size-3.5 shrink-0 text-muted-foreground/50" />
          )}
          {isVolume && (
            <FolderOpen className={cn(
              "size-3.5 shrink-0 transition-colors",
              expanded ? "text-primary/60" : "text-muted-foreground/50",
            )} />
          )}
          {editingId === node.id ? (
            <Input
              value={editingTitle}
              onChange={(e) => setEditingTitle(e.target.value)}
              onBlur={() => finishEditing(node.id, parentId)}
              onKeyDown={(e) => {
                if (e.key === "Enter") finishEditing(node.id, parentId);
                if (e.key === "Escape") setEditingId(null);
              }}
              className="flex-1 h-6 min-w-0 text-sm"
              autoFocus
              onClick={(e) => e.stopPropagation()}
            />
          ) : (
            <span className="flex-1 truncate">{node.title}</span>
          )}
          {node.node_type === "chapter" && node.word_count > 0 && !editingId && (
            <span className="text-[10px] text-muted-foreground/40 tabular-nums shrink-0">{node.word_count}</span>
          )}
          <div className="relative flex items-center opacity-0 group-hover:opacity-100 transition-opacity">
            {isVolume && (
              <Button
                variant="ghost"
                size="icon-xs"
                onClick={(e) => {
                  e.stopPropagation();
                  handleAddChapter(node.id);
                }}
              >
                <Plus />
              </Button>
            )}
            <DropdownMenu>
              <DropdownMenuTrigger
                render={(props) => (
                  <Button variant="ghost" size="icon-xs" {...props}>
                    <MoreHorizontal />
                  </Button>
                )}
                onClick={(e) => e.stopPropagation()}
              />
              <DropdownMenuContent align="end" side="right">
                <DropdownMenuGroup>
                  <DropdownMenuItem
                    onClick={() => {
                      setEditingId(node.id);
                      setEditingTitle(node.title);
                    }}
                  >
                    <Pencil className="size-3" />
                    重命名
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    variant="destructive"
                    onClick={() => handleDelete(node.id, parentId)}
                  >
                    <Trash2 className="size-3" />
                    删除
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
        <CollapsibleContent>
          <div>
            {children.map((child) => renderNode(child, node.id, depth + 1))}
            {children.length === 0 && (
              <p
                className="py-1.5 text-[11px] text-muted-foreground/40"
                style={{ paddingLeft: `${(depth + 1) * 14 + 22}px` }}
              >
                暂无章节
              </p>
            )}
          </div>
        </CollapsibleContent>
      </Collapsible>
    );
  };

  return (
    <div className="flex flex-col gap-0.5 p-2">
      <div className="relative mb-1">
        <Search className="absolute left-2 top-1/2 -translate-y-1/2 size-3 text-muted-foreground/40" />
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="搜索..."
          className="h-7 pl-7 text-xs bg-secondary/30 border-0 focus-visible:border-primary/30"
        />
      </div>
      {filteredNodes.map((node) => renderNode(node, null))}
      <DropdownMenu>
        <DropdownMenuTrigger
          render={
            <button
              type="button"
              className="mt-1 flex w-full items-center justify-center gap-1.5 rounded-md px-2 py-1.5 text-xs text-muted-foreground/50 transition-colors hover:bg-secondary/50 hover:text-muted-foreground"
            >
              <Plus className="size-3" />
              新建
            </button>
          }
        />
        <DropdownMenuContent align="start" side="top">
          <DropdownMenuGroup>
            <DropdownMenuItem onClick={() => handleAddChapter(null)}>
              <FileText className="size-3.5" />
              新建章节
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => handleAddVolume()}>
              <BookOpen className="size-3.5" />
              新建卷
            </DropdownMenuItem>
          </DropdownMenuGroup>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};

export default ChapterTree;
