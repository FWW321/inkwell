import { useState, useCallback, useRef, useEffect } from "react";
import { Outlet, useParams, Link, useLocation, useNavigate } from "react-router-dom";
import { AnimatePresence, motion } from "motion/react";
import {
  PanelLeftClose,
  PanelLeft,
  PenLine,
  Users,
  Globe,
  Settings,
  Feather,
  BookOpen,
  BookMarked,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";
import ChapterTree from "@/components/chapter/ChapterTree";
import { projectApi } from "@/lib/api";
import { springs } from "@/lib/motion";

const navItems = [
  { path: "write", label: "写作", icon: PenLine },
  { path: "characters", label: "角色", icon: Users },
  { path: "worldview", label: "世界观", icon: Globe },
  { path: "details", label: "详情", icon: BookMarked },
] as const;

const SIDEBAR_MIN = 180;
const SIDEBAR_DEFAULT = 260;
const SIDEBAR_MAX = 400;

const ProjectLayout = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const { pathname } = useLocation();
  const navigate = useNavigate();
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [sidebarWidth, setSidebarWidth] = useState(SIDEBAR_DEFAULT);
  const [projectName, setProjectName] = useState("");
  const isResizing = useRef(false);
  const resizeStartX = useRef(0);
  const resizeStartWidth = useRef(0);

  const activeNav = navItems.find((item) =>
    pathname.includes(`/project/${projectId}/${item.path}`),
  );

  const showSidebar = sidebarOpen && activeNav?.path === "write";

  useEffect(() => {
    if (!projectId) return;
    projectApi.get(projectId).then((p) => setProjectName(p.title)).catch(() => {});
  }, [projectId]);

  const handleNavClick = useCallback(
    (path: string) => {
      navigate(`/project/${projectId}/${path}`);
    },
    [navigate, projectId],
  );

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isResizing.current = true;
    resizeStartX.current = e.clientX;
    resizeStartWidth.current = sidebarWidth;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
  }, [sidebarWidth]);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing.current) return;
      const delta = e.clientX - resizeStartX.current;
      const next = Math.max(SIDEBAR_MIN, Math.min(SIDEBAR_MAX, resizeStartWidth.current + delta));
      setSidebarWidth(next);
    };

    const handleMouseUp = () => {
      if (!isResizing.current) return;
      isResizing.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, []);

  return (
    <div className="flex h-screen overflow-hidden">
      <aside className="flex h-full w-12 shrink-0 flex-col items-center border-r border-sidebar-border bg-sidebar py-2 gap-1">
        <Tooltip>
          <TooltipTrigger
            render={
              <Link
                to="/"
                className="flex size-8 items-center justify-center rounded-lg text-sidebar-foreground/70 transition-colors hover:bg-sidebar-accent hover:text-sidebar-foreground"
              >
                <Feather className="size-4" />
              </Link>
            }
          />
          <TooltipContent side="right">返回项目</TooltipContent>
        </Tooltip>

        <div className="mx-2 my-1 h-px w-5 bg-sidebar-border" />

        {navItems.map((item) => {
          const isActive = activeNav?.path === item.path;
          return (
            <Tooltip key={item.path}>
              <TooltipTrigger
                render={
                  <button
                    type="button"
                    onClick={() => handleNavClick(item.path)}
                    className={cn(
                      "flex size-8 items-center justify-center rounded-lg transition-colors",
                      isActive
                        ? "bg-primary/15 text-primary"
                        : "text-sidebar-foreground/50 hover:bg-sidebar-accent hover:text-sidebar-foreground",
                    )}
                  >
                    <item.icon className="size-4" />
                  </button>
                }
              />
              <TooltipContent side="right">{item.label}</TooltipContent>
            </Tooltip>
          );
        })}

        <div className="flex-1" />

        <Tooltip>
          <TooltipTrigger
            render={
              <Link
                to="/settings"
                className={cn(
                  "flex size-8 items-center justify-center rounded-lg transition-colors",
                  pathname === "/settings"
                    ? "bg-primary/15 text-primary"
                    : "text-sidebar-foreground/50 hover:bg-sidebar-accent hover:text-sidebar-foreground",
                )}
              >
                <Settings className="size-4" />
              </Link>
            }
          />
          <TooltipContent side="right">设置</TooltipContent>
        </Tooltip>

        {activeNav?.path === "write" && (
          <Tooltip>
            <TooltipTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon-xs"
                  onClick={() => setSidebarOpen(!sidebarOpen)}
                  className="text-sidebar-foreground/50 hover:text-sidebar-foreground hover:bg-sidebar-accent"
                >
                  {sidebarOpen ? <PanelLeftClose className="size-3.5" /> : <PanelLeft className="size-3.5" />}
                </Button>
              }
            />
            <TooltipContent side="right">{sidebarOpen ? "收起侧栏" : "展开侧栏"}</TooltipContent>
          </Tooltip>
        )}
      </aside>

      <AnimatePresence initial={false}>
        {showSidebar && (
          <>
            <motion.aside
              initial={{ width: 0, opacity: 0 }}
              animate={{ width: sidebarWidth, opacity: 1 }}
              exit={{ width: 0, opacity: 0 }}
              transition={springs.smooth}
              className="flex h-full shrink-0 flex-col overflow-hidden border-r border-border bg-background"
            >
              <div className="flex items-center gap-2 px-4 h-9 shrink-0 border-b border-border">
                <BookOpen className="size-3.5 text-primary/50 shrink-0" />
                <span className="text-xs font-medium text-foreground/80 truncate">{projectName}</span>
              </div>
              <div className="flex-1 overflow-y-auto">
                {projectId && <ChapterTree projectId={projectId} />}
              </div>
            </motion.aside>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.15 }}
              className="w-px shrink-0 cursor-col-resize bg-transparent hover:bg-primary/30 active:bg-primary/50 transition-colors"
              onMouseDown={handleMouseDown}
            />
          </>
        )}
      </AnimatePresence>

      <main className="flex-1 overflow-hidden bg-background flex flex-col">
        {projectName && activeNav?.path !== "write" && (
          <div className="flex items-center gap-2 border-b border-border px-5 h-8 shrink-0">
            <BookOpen className="size-3 text-primary/40 shrink-0" />
            <span className="text-xs text-muted-foreground/60 truncate">{projectName}</span>
          </div>
        )}
        <AnimatePresence mode="wait">
          <motion.div
            key={pathname}
            className="flex-1 overflow-hidden"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.12 }}
          >
            <Outlet />
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
};

export default ProjectLayout;
