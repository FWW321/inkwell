import { useState, useEffect } from "react";
import { motion } from "motion/react";
import { useNavigate } from "react-router-dom";
import { Plus, BookOpen, Trash2, MoreHorizontal, Feather, Settings } from "lucide-react";
import { projectApi } from "@/lib/api";
import { cn } from "@/lib/utils";
import type { Project, ProjectStatus } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Select, SelectTrigger, SelectContent, SelectGroup, SelectItem, SelectValue } from "@/components/ui/select";
import { Spinner } from "@/components/ui/spinner";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription, EmptyContent } from "@/components/ui/empty";
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuGroup, DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog";
import { springs } from "@/lib/motion";

const languages = [
  { value: "zh", label: "中文" },
  { value: "zh-TW", label: "繁體中文" },
  { value: "en", label: "English" },
  { value: "ja", label: "日本語" },
  { value: "ko", label: "한국어" },
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
  { value: "pt", label: "Português" },
  { value: "ru", label: "Русский" },
] as const;

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
  const [author, setAuthor] = useState("");
  const [language, setLanguage] = useState("");
  const [tags, setTags] = useState("");
  const [status, setStatus] = useState<ProjectStatus>("ongoing");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    setLoading(true);
    try {
      const project = await projectApi.create(title.trim(), description.trim(), author.trim(), language.trim(), tags.trim(), status);
      onCreated(project);
      setTitle("");
      setDescription("");
      setAuthor("");
      setLanguage("");
      setTags("");
      setStatus("ongoing");
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
            <Label>书名</Label>
            <Input
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="输入书名..."
              autoFocus
            />
          </div>
          <div className="flex gap-3">
            <div className="flex flex-col gap-1.5 flex-1">
              <Label>作者</Label>
              <Input
                value={author}
                onChange={(e) => setAuthor(e.target.value)}
                placeholder="作者名..."
              />
            </div>
            <div className="flex flex-col gap-1.5 flex-1">
              <Label>语言</Label>
              <Select value={language} onValueChange={(v) => setLanguage(v ?? "")}>
                <SelectTrigger>
                  <SelectValue placeholder="选择语言">{language ? languages.find((l) => l.value === language)?.label : "选择语言"}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    {languages.map((l) => (
                      <SelectItem key={l.value} value={l.value}>{l.label}</SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="flex flex-col gap-1.5">
            <Label>标签</Label>
            <Input
              value={tags}
              onChange={(e) => setTags(e.target.value)}
              placeholder="用逗号分隔，如：奇幻,冒险,热血"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label>状态</Label>
            <div className="flex gap-2">
              {([
                { value: "ongoing" as const, label: "连载中" },
                { value: "completed" as const, label: "已完结" },
                { value: "hiatus" as const, label: "暂停" },
              ]).map((s) => (
                <button
                  key={s.value}
                  type="button"
                  onClick={() => setStatus(s.value)}
                  className={cn(
                    "rounded-md px-3 py-1.5 text-xs transition-colors border",
                    status === s.value
                      ? "bg-primary/15 border-primary/30 text-primary"
                      : "border-border text-muted-foreground hover:bg-secondary",
                  )}
                >
                  {s.label}
                </button>
              ))}
            </div>
          </div>
          <div className="flex flex-col gap-1.5">
            <Label>简介</Label>
            <Textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="简要描述你的故事..."
              rows={3}
            />
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>取消</Button>
            <Button type="submit" disabled={loading || !title.trim()}>
              {loading ? "创建中..." : "创建项目"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
};

const coverColors = [
  { gradient: "from-violet-600/25 to-indigo-900/40", accent: "text-violet-300", border: "border-violet-500/10", spine: "from-violet-800/30 to-violet-600/20" },
  { gradient: "from-cyan-600/25 to-teal-900/40", accent: "text-cyan-300", border: "border-cyan-500/10", spine: "from-cyan-800/30 to-cyan-600/20" },
  { gradient: "from-amber-600/25 to-orange-900/40", accent: "text-amber-300", border: "border-amber-500/10", spine: "from-amber-800/30 to-amber-600/20" },
  { gradient: "from-rose-600/25 to-pink-900/40", accent: "text-rose-300", border: "border-rose-500/10", spine: "from-rose-800/30 to-rose-600/20" },
  { gradient: "from-emerald-600/25 to-green-900/40", accent: "text-emerald-300", border: "border-emerald-500/10", spine: "from-emerald-800/30 to-emerald-600/20" },
  { gradient: "from-blue-600/25 to-sky-900/40", accent: "text-blue-300", border: "border-blue-500/10", spine: "from-blue-800/30 to-blue-600/20" },
] as const;

const getCoverStyle = (id: string) => {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = ((hash << 5) - hash + id.charCodeAt(i)) | 0;
  }
  return coverColors[Math.abs(hash) % coverColors.length];
};

const ProjectCard = ({
  project,
  onDelete,
}: {
  project: Project;
  onDelete: (id: string) => void;
}) => {
  const navigate = useNavigate();
  const cover = getCoverStyle(project.id);
  const displayTags = project.tags ? project.tags.split(",").map((t) => t.trim()).filter(Boolean) : [];

  const statusLabel: Record<ProjectStatus, string> = {
    ongoing: "连载中",
    completed: "已完结",
    hiatus: "暂停",
  };

  const displayTitle = project.title.length > 6 ? project.title.slice(0, 6) + "…" : project.title;

  return (
    <motion.div
      className="group/card flex flex-col cursor-pointer"
      onClick={() => navigate(`/project/${project.id}/write`)}
      whileHover={{ y: -2 }}
      transition={springs.snappy}
    >
      <div className={cn(
        "relative aspect-[3/4] rounded-lg overflow-hidden",
        "shadow-sm hover:shadow-lg hover:shadow-primary/5",
      )}>
        <div className={cn("absolute inset-0 bg-gradient-to-br", cover.gradient)} />
        <div className="absolute left-0 top-0 bottom-0 w-3 bg-gradient-to-r" style={{
          backgroundImage: `linear-gradient(to right, rgba(0,0,0,0.25), rgba(0,0,0,0.05), transparent)`,
        }} />
        <div className="relative flex h-full flex-col items-center justify-between p-4 pt-5 pb-3.5">
          <div className="flex flex-col items-center gap-2">
            <BookOpen className={cn("size-6", cover.accent)} />
          </div>
          <div className="flex flex-col items-center gap-1.5 text-center flex-1 justify-center">
            <p className={cn("text-base font-semibold tracking-wide leading-tight", cover.accent)}>
              {displayTitle}
            </p>
            <div className={cn("w-6 h-px", cover.accent, "opacity-30")} />
            {project.author && (
              <p className="text-[10px] text-white/40">{project.author}</p>
            )}
          </div>
          <div className="flex flex-col items-center gap-1.5">
            {project.status !== "ongoing" && (
              <span className="rounded px-2 py-0.5 text-[9px] bg-white/10 text-white/50">{statusLabel[project.status]}</span>
            )}
            {displayTags.length > 0 && (
              <div className="flex flex-wrap items-center justify-center gap-1 max-w-full">
                {displayTags.slice(0, 3).map((tag) => (
                  <span key={tag} className="rounded px-1.5 py-0.5 text-[9px] text-white/30 bg-white/5">{tag}</span>
                ))}
                {displayTags.length > 3 && (
                  <span className="text-[9px] text-white/20">+{displayTags.length - 3}</span>
                )}
              </div>
            )}
          </div>
        </div>
        <div className="absolute top-2 right-2">
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="opacity-0 group-hover/card:opacity-100 text-white/50 hover:text-white/80 hover:bg-white/10"
                  onClick={(e: React.MouseEvent) => e.stopPropagation()}
                />
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
        </div>
      </div>
      <div className="mt-2 px-0.5">
        <p className="text-center text-xs font-medium text-foreground truncate">{project.title}</p>
        {(project.author || project.language) && (
          <p className="text-center text-[10px] text-muted-foreground/50 truncate">
            {[project.author, project.language].filter(Boolean).join(" · ")}
          </p>
        )}
      </div>
    </motion.div>
  );
};

const ProjectsPage = () => {
  const navigate = useNavigate();
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
      <div className="flex items-center justify-between px-8 py-6">
        <div className="flex items-center gap-3">
          <div className="flex size-8 items-center justify-center rounded-lg bg-primary/12">
            <Feather className="size-4 text-primary" />
          </div>
          <h1 className="text-lg font-semibold text-foreground">Inkwell</h1>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon-sm"
            className="text-muted-foreground/60"
            onClick={() => navigate("/settings")}
          >
            <Settings />
          </Button>
          <Button onClick={() => setShowCreate(true)}>
            <Plus data-icon="inline-start" />
            新建项目
          </Button>
        </div>
      </div>
      <div className="flex-1 overflow-y-auto px-8 pb-8">
        {loading ? (
          <div className="flex h-64 items-center justify-center">
            <div className="flex flex-col items-center gap-3">
              <Spinner className="size-5" />
              <p className="text-sm text-muted-foreground">加载中...</p>
            </div>
          </div>
        ) : projects.length === 0 ? (
          <Empty className="h-[calc(100vh-200px)]">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <Feather className="size-10 text-primary/60" />
              </EmptyMedia>
              <EmptyTitle className="text-base">还没有项目</EmptyTitle>
              <EmptyDescription>创建你的第一个小说项目</EmptyDescription>
            </EmptyHeader>
            <EmptyContent>
              <Button onClick={() => setShowCreate(true)}>
                <Plus data-icon="inline-start" />
                新建项目
              </Button>
            </EmptyContent>
          </Empty>
        ) : (
          <div className="grid grid-cols-2 gap-5 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
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
