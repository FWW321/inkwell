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
import ChapterTree from "@/components/chapter/ChapterTree";

const navItems = [
  { icon: PenLine, label: "编辑", path: "write" },
  { icon: Users, label: "角色", path: "characters" },
  { icon: Globe, label: "世界观", path: "worldview" },
  { icon: Settings, label: "设置", path: "settings" },
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
              <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary/15">
                <Feather className="h-3.5 w-3.5 text-primary" />
              </div>
              Inkwell
            </Link>
          )}
          <button
            onClick={() => setSidebarOpen(!sidebarOpen)}
            className={cn(
              "flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-all hover:bg-secondary hover:text-foreground",
              !sidebarOpen && "mx-auto",
            )}
          >
            {sidebarOpen ? (
              <PanelLeftClose className="h-4 w-4" />
            ) : (
              <PanelLeft className="h-4 w-4" />
            )}
          </button>
        </div>
        {sidebarOpen && projectId && (
          <div className="flex-1 overflow-y-auto">
            <ChapterTree projectId={projectId} />
          </div>
        )}
        <nav className="border-t border-border p-1.5">
          {navItems.map((item) => {
            const isActive = activeNav?.path === item.path;
            return (
              <Link
                key={item.path}
                to={`/project/${projectId}/${item.path}`}
                className={cn(
                  "flex items-center gap-2.5 rounded-lg px-2.5 py-2 text-sm transition-all",
                  !sidebarOpen && "justify-center px-0",
                  isActive
                    ? "bg-primary/12 text-primary font-medium"
                    : "text-sidebar-foreground/70 hover:bg-secondary hover:text-sidebar-foreground",
                )}
                title={item.label}
              >
                <item.icon className={cn("h-4 w-4 shrink-0", isActive && "text-primary")} />
                {sidebarOpen && <span>{item.label}</span>}
              </Link>
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
