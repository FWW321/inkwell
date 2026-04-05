import { useState, useEffect } from "react";
import { useNavigate, useParams } from "react-router-dom";
import {
  ChevronRight,
  Plus,
  FileText,
  FolderOpen,
  MoreHorizontal,
  Trash2,
  Pencil,
} from "lucide-react";
import { outlineApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { OutlineNode } from "@/lib/types";

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
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

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
      const node = await outlineApi.create(projectId, null, "volume", `第${nodes.length + 1}卷`);
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
      const chap = await outlineApi.create(
        projectId,
        parentId,
        "chapter",
        `第${siblings.length + 1}章`,
      );
      if (parentId) {
        setChildrenMap((prev) => ({
          ...prev,
          [parentId]: [...(prev[parentId] || []), chap],
        }));
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
      <div className="p-3">
        <div className="h-4 w-16 animate-pulse-subtle rounded bg-muted" />
      </div>
    );
  }

  const renderNode = (node: OutlineNode, parentId: string | null, depth: number = 0) => {
    const isVolume = node.node_type === "volume";
    const expanded = expandedIds.has(node.id);
    const children = childrenMap[node.id] || [];
    const isActive = chapterId === node.id;

    return (
      <div key={node.id}>
        <div
          className={cn(
            "group flex items-center gap-1 rounded-lg px-2 py-1.5 text-[13px] transition-all cursor-pointer",
            isActive
              ? "bg-primary/12 text-primary font-medium"
              : "text-sidebar-foreground/80 hover:bg-secondary/60 hover:text-sidebar-foreground",
          )}
          style={{ paddingLeft: `${depth * 16 + 8}px` }}
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
                "h-3 w-3 shrink-0 text-muted-foreground transition-transform duration-200",
                expanded && "rotate-90",
              )}
            />
          ) : (
            <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground/70" />
          )}
          {isVolume && (
            <FolderOpen className={cn(
              "h-3.5 w-3.5 shrink-0 transition-colors",
              expanded ? "text-primary/70" : "text-muted-foreground/60",
            )} />
          )}
          {editingId === node.id ? (
            <input
              value={editingTitle}
              onChange={(e) => setEditingTitle(e.target.value)}
              onBlur={() => finishEditing(node.id, parentId)}
              onKeyDown={(e) => {
                if (e.key === "Enter") finishEditing(node.id, parentId);
                if (e.key === "Escape") setEditingId(null);
              }}
              className="flex-1 bg-transparent text-sm outline-none min-w-0"
              autoFocus
              onClick={(e) => e.stopPropagation()}
            />
          ) : (
            <span className="flex-1 truncate">{node.title}</span>
          )}
          <div className="relative flex items-center opacity-0 transition-opacity group-hover:opacity-100">
            {isVolume && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleAddChapter(node.id);
                }}
                className="rounded p-0.5 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
                title="添加章节"
              >
                <Plus className="h-3 w-3" />
              </button>
            )}
            <button
              onClick={(e) => {
                e.stopPropagation();
                setMenuOpenId(menuOpenId === node.id ? null : node.id);
              }}
              className="rounded p-0.5 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground"
            >
              <MoreHorizontal className="h-3 w-3" />
            </button>
            {menuOpenId === node.id && (
              <div className="animate-fade-in absolute right-0 top-7 z-20 w-28 rounded-lg border border-border bg-card py-1 shadow-xl">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setEditingId(node.id);
                    setEditingTitle(node.title);
                    setMenuOpenId(null);
                  }}
                  className="flex w-full items-center gap-2 px-2.5 py-1.5 text-xs transition-colors hover:bg-secondary"
                >
                  <Pencil className="h-3 w-3" />
                  重命名
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(node.id, parentId);
                    setMenuOpenId(null);
                  }}
                  className="flex w-full items-center gap-2 px-2.5 py-1.5 text-xs text-destructive transition-colors hover:bg-destructive/10"
                >
                  <Trash2 className="h-3 w-3" />
                  删除
                </button>
              </div>
            )}
          </div>
        </div>
        {isVolume && expanded && (
          <div>
            {children.map((child) => renderNode(child, node.id, depth + 1))}
            {children.length === 0 && (
              <p
                className="py-1.5 text-[11px] text-muted-foreground/50"
                style={{ paddingLeft: `${(depth + 1) * 16 + 24}px` }}
              >
                暂无章节
              </p>
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="flex flex-col gap-0.5 p-2">
      <p className="px-2 py-1 text-[11px] font-medium uppercase tracking-wider text-muted-foreground/60">
        大纲
      </p>
      {nodes.map((node) => renderNode(node, null))}
      <div className="mt-1 flex gap-1 px-1">
        <button
          onClick={() => handleAddChapter(null)}
          className="flex items-center gap-1.5 rounded-lg px-2 py-1.5 text-[12px] text-muted-foreground/70 transition-all hover:bg-secondary hover:text-muted-foreground"
        >
          <Plus className="h-3 w-3" />
          章节目录
        </button>
        <button
          onClick={handleAddVolume}
          className="flex items-center gap-1.5 rounded-lg px-2 py-1.5 text-[12px] text-muted-foreground/70 transition-all hover:bg-secondary hover:text-muted-foreground"
        >
          <Plus className="h-3 w-3" />
          添加卷
        </button>
      </div>
    </div>
  );
};

export default ChapterTree;
