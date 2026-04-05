import { useState } from "react";
import { Outlet, useParams, Link, useLocation } from "react-router-dom";
import {
  PanelLeftClose,
  PanelLeft,
  Settings,
  Users,
  Globe,
  PenLine,
  Feather,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Tooltip, TooltipTrigger, TooltipContent } from "@/components/ui/tooltip";
import ChapterTree from "@/components/chapter/ChapterTree";

const navItems = [
  { path: "write", label: "写作", icon: PenLine },
  { path: "characters", label: "角色", icon: Users },
  { path: "worldview", label: "世界观", icon: Globe },
  { path: "settings", label: "设置", icon: Settings },
];

const ProjectLayout = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const { pathname } = useLocation();
  const [sidebarOpen, setSidebarOpen] = useState(true);

  const activeNav = navItems.find((item) =>
    pathname.includes(`/project/${projectId}/${item.path}`),
  );

  return (
    <div className="flex h-screen">
      <aside
        className={cn(
          "flex flex-col border-r border-border bg-sidebar transition-all duration-200 ease-out",
          sidebarOpen ? "w-60" : "w-14",
        )}
      >
        <div className="flex items-center gap-2 border-b border-border px-3 py-3">
          {sidebarOpen && (
            <Link
              to="/"
              className="flex items-center gap-2 text-sm font-semibold text-sidebar-foreground transition-colors hover:text-foreground"
            >
              <div className="flex size-7 items-center justify-center rounded-md bg-primary/15">
                <Feather className="size-3.5 text-primary" />
              </div>
              Inkwell
            </Link>
          )}
          <Button
            variant="ghost"
            size="icon-sm"
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className={cn(!sidebarOpen && "mx-auto")}
          >
            {sidebarOpen ? (
              <PanelLeftClose />
            ) : (
              <PanelLeft />
            )}
          </Button>
        </div>
        {sidebarOpen && projectId && (
          <div className="flex-1 overflow-y-auto">
            <ChapterTree projectId={projectId} />
          </div>
        )}
        <Separator />
        <nav className="p-1.5">
          {navItems.map((item) => {
            const isActive = activeNav?.path === item.path;
            return (
              <Tooltip key={item.path}>
                <TooltipTrigger
                  render={
                    <Link
                      to={`/project/${projectId}/${item.path}`}
                      className={cn(
                        "flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition-all",
                        !sidebarOpen && "justify-center px-0",
                        isActive
                          ? "bg-primary/12 text-primary font-medium"
                          : "text-sidebar-foreground/70 hover:bg-secondary hover:text-sidebar-foreground",
                      )}
                    />
                  }
                >
                  <item.icon className={cn("size-4 shrink-0", isActive && "text-primary")} />
                  {sidebarOpen && <span>{item.label}</span>}
                </TooltipTrigger>
                {!sidebarOpen && (
                  <TooltipContent side="right">{item.label}</TooltipContent>
                )}
              </Tooltip>
            );
          })}
        </nav>
      </aside>
      <main className="flex-1 overflow-hidden bg-background">
        <Outlet />
      </main>
    </div>
  );
};

export default ProjectLayout;
