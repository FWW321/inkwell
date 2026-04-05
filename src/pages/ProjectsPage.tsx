import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Plus, BookOpen, Trash2, MoreHorizontal, Feather } from "lucide-react";
import { projectApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { Project } from "@/lib/types";

const CreateProjectDialog = ({
  open,
  onClose,
  onCreated,
}: {
  open: boolean;
  onClose: () => void;
  onCreated: (project: Project) => void;
}) => {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [loading, setLoading] = useState(false);

  if (!open) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    setLoading(true);
    try {
      const project = await projectApi.create(title.trim(), description.trim());
      onCreated(project);
      setTitle("");
      setDescription("");
      onClose();
    } catch (err) {
      console.error("Failed to create project:", err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={onClose}>
      <div className="animate-fade-in w-full max-w-md rounded-xl border border-border bg-card p-6 shadow-2xl" onClick={(e) => e.stopPropagation()}>
        <h2 className="mb-1 text-lg font-semibold text-foreground">新建项目</h2>
        <p className="mb-5 text-sm text-muted-foreground">开始你的下一个故事</p>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="mb-1.5 block text-sm font-medium text-foreground">
              项目名称
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="输入项目名称..."
              className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/60 focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
              autoFocus
            />
          </div>
          <div>
            <label className="mb-1.5 block text-sm font-medium text-foreground">
              简介
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="简要描述你的故事..."
              rows={3}
              className="w-full rounded-lg border border-input bg-background px-3 py-2.5 text-sm outline-none placeholder:text-muted-foreground/60 focus:border-primary focus:ring-1 focus:ring-primary transition-colors resize-none"
            />
          </div>
          <div className="flex justify-end gap-2 pt-1">
            <button
              type="button"
              onClick={onClose}
              className="rounded-lg px-4 py-2.5 text-sm text-muted-foreground transition-colors hover:bg-secondary hover:text-secondary-foreground"
            >
              取消
            </button>
            <button
              type="submit"
              disabled={loading || !title.trim()}
              className={cn(
                "rounded-lg px-5 py-2.5 text-sm font-medium text-primary-foreground transition-all",
                "bg-primary hover:bg-primary/90 active:scale-[0.97] disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100",
              )}
            >
              {loading ? "创建中..." : "创建项目"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

const ProjectCard = ({
  project,
  onDelete,
}: {
  project: Project;
  onDelete: (id: string) => void;
}) => {
  const navigate = useNavigate();
  const [showMenu, setShowMenu] = useState(false);

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString("zh-CN", {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
    } catch {
      return dateStr;
    }
  };

  return (
    <div
      className="group relative cursor-pointer rounded-xl border border-border bg-card p-5 transition-all duration-200 hover:border-primary/30 hover:bg-card/80 active:scale-[0.98]"
      onClick={() => navigate(`/project/${project.id}/write`)}
    >
      <div className="mb-3 flex items-start justify-between">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/15">
            <BookOpen className="h-5 w-5 text-primary" />
          </div>
          <div>
            <h3 className="font-semibold text-foreground">{project.title}</h3>
            {project.description && (
              <p className="mt-0.5 line-clamp-1 text-sm text-muted-foreground">
                {project.description}
              </p>
            )}
          </div>
        </div>
        <div className="relative">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setShowMenu(!showMenu);
            }}
            className="rounded-md p-1.5 text-muted-foreground opacity-0 transition-all hover:bg-secondary group-hover:opacity-100"
          >
            <MoreHorizontal className="h-4 w-4" />
          </button>
          {showMenu && (
            <div className="animate-fade-in absolute right-0 top-8 z-10 w-32 rounded-lg border border-border bg-card py-1 shadow-xl">
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(project.id);
                  setShowMenu(false);
                }}
                className="flex w-full items-center gap-2 px-3 py-2 text-sm text-destructive transition-colors hover:bg-destructive/10"
              >
                <Trash2 className="h-3.5 w-3.5" />
                删除项目
              </button>
            </div>
          )}
        </div>
      </div>
      <p className="text-xs text-muted-foreground/70">
        更新于 {formatDate(project.updated_at)}
      </p>
    </div>
  );
};

const ProjectsPage = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [showCreate, setShowCreate] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const loadProjects = async () => {
      try {
        const data = await projectApi.list();
        setProjects(data);
      } catch (err) {
        console.error("Failed to load projects:", err);
      } finally {
        setLoading(false);
      }
    };
    loadProjects();
  }, []);

  const handleDelete = async (id: string) => {
    try {
      await projectApi.delete(id);
      setProjects((prev) => prev.filter((p) => p.id !== id));
    } catch (err) {
      console.error("Failed to delete project:", err);
    }
  };

  const handleCreated = (project: Project) => {
    setProjects((prev) => [project, ...prev]);
  };

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-border px-8 py-5">
        <div className="flex items-center gap-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/15">
            <Feather className="h-5 w-5 text-primary" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-foreground tracking-tight">Inkwell</h1>
            <p className="text-xs text-muted-foreground">AI 驱动的小说创作工具</p>
          </div>
        </div>
        <button
          onClick={() => setShowCreate(true)}
          className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2.5 text-sm font-medium text-primary-foreground transition-all hover:bg-primary/90 active:scale-[0.97]"
        >
          <Plus className="h-4 w-4" />
          新建项目
        </button>
      </header>
      <div className="flex-1 overflow-y-auto px-8 py-6">
        {loading ? (
          <div className="flex h-64 items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
              <p className="text-sm text-muted-foreground">加载中...</p>
            </div>
          </div>
        ) : projects.length === 0 ? (
          <div className="flex h-[calc(100vh-200px)] flex-col items-center justify-center gap-5">
            <div className="flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10">
              <Feather className="h-10 w-10 text-primary/70" />
            </div>
            <div className="text-center">
              <p className="text-lg font-semibold text-foreground">还没有项目</p>
              <p className="mt-1 text-sm text-muted-foreground">创建你的第一个小说项目，开始创作之旅</p>
            </div>
            <button
              onClick={() => setShowCreate(true)}
              className="flex items-center gap-2 rounded-lg bg-primary px-5 py-2.5 text-sm font-medium text-primary-foreground transition-all hover:bg-primary/90 active:scale-[0.97]"
            >
              <Plus className="h-4 w-4" />
              新建项目
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {projects.map((project) => (
              <ProjectCard
                key={project.id}
                project={project}
                onDelete={handleDelete}
              />
            ))}
          </div>
        )}
      </div>
      <CreateProjectDialog
        open={showCreate}
        onClose={() => setShowCreate(false)}
        onCreated={handleCreated}
      />
    </div>
  );
};

export default ProjectsPage;
