import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { Plus, BookOpen, Trash2, MoreHorizontal, Feather } from "lucide-react";
import { projectApi } from "@/lib/api";
import type { Project } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Spinner } from "@/components/ui/spinner";
import { Card, CardContent, CardHeader, CardTitle, CardDescription, CardAction } from "@/components/ui/card";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from "@/components/ui/empty";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";

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
    <Dialog open={open} onOpenChange={(isOpen) => { if (!isOpen) onClose(); }}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>新建项目</DialogTitle>
          <DialogDescription>开始你的下一个故事</DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1.5">
            <label className="text-sm font-medium text-foreground">项目名称</label>
            <Input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="输入项目名称..."
              autoFocus
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-sm font-medium text-foreground">简介</label>
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="简要描述你的故事..."
              rows={3}
            />
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
              取消
            </Button>
            <Button type="submit" disabled={loading || !title.trim()}>
              {loading ? "创建中..." : "创建项目"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
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
    <Card
      className="cursor-pointer transition-all duration-200 hover:border-primary/30 active:scale-[0.98]"
      onClick={() => navigate(`/project/${project.id}/write`)}
    >
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="flex size-10 items-center justify-center rounded-lg bg-primary/15">
            <BookOpen className="size-5 text-primary" />
          </div>
          <div className="flex-1 min-w-0">
            <CardTitle>{project.title}</CardTitle>
            {project.description && (
              <CardDescription className="mt-0.5 line-clamp-1">
                {project.description}
              </CardDescription>
            )}
          </div>
        </div>
        <CardAction>
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button variant="ghost" size="icon-sm" className="opacity-0 group-hover/card:opacity-100" />
              }
            >
              <MoreHorizontal />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuGroup>
                <DropdownMenuItem
                  variant="destructive"
                  onClick={(e: React.MouseEvent) => {
                    e.stopPropagation();
                    onDelete(project.id);
                  }}
                >
                  <Trash2 />
                  删除项目
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </CardAction>
      </CardHeader>
      <CardContent>
        <p className="text-xs text-muted-foreground/70">
          更新于 {formatDate(project.updated_at)}
        </p>
      </CardContent>
    </Card>
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
          <div className="flex size-9 items-center justify-center rounded-lg bg-primary/15">
            <Feather className="size-5 text-primary" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-foreground tracking-tight">Inkwell</h1>
            <p className="text-xs text-muted-foreground">AI 驱动的小说创作工具</p>
          </div>
        </div>
        <Button onClick={() => setShowCreate(true)}>
          <Plus data-icon="inline-start" />
          新建项目
        </Button>
      </header>
      <div className="flex-1 overflow-y-auto px-8 py-6">
        {loading ? (
          <div className="flex h-64 items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <Spinner className="size-6" />
              <p className="text-sm text-muted-foreground">加载中...</p>
            </div>
          </div>
        ) : projects.length === 0 ? (
          <Empty className="h-[calc(100vh-200px)]">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Feather className="size-10 text-primary/70" />
              </EmptyMedia>
              <EmptyTitle className="text-lg">还没有项目</EmptyTitle>
              <EmptyDescription>创建你的第一个小说项目，开始创作之旅</EmptyDescription>
            </EmptyHeader>
            <EmptyContent>
              <Button onClick={() => setShowCreate(true)}>
                <Plus data-icon="inline-start" />
                新建项目
              </Button>
            </EmptyContent>
          </Empty>
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
