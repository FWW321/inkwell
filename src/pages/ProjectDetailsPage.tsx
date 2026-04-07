import { useState, useEffect } from "react";
import { useParams } from "react-router-dom";
import { Save, Check } from "lucide-react";
import { projectApi } from "@/lib/api";
import type { Project, ProjectStatus } from "@/lib/types";
import { cn } from "@/lib/utils";
import { languages } from "@/lib/constants";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Spinner } from "@/components/ui/spinner";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Select, SelectTrigger, SelectContent, SelectGroup, SelectItem, SelectValue } from "@/components/ui/select";

const statusOptions: { value: ProjectStatus; label: string }[] = [
  { value: "ongoing", label: "连载中" },
  { value: "completed", label: "已完结" },
  { value: "hiatus", label: "暂停" },
];

const ProjectDetailsPage = () => {
  const { projectId } = useParams<{ projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [author, setAuthor] = useState("");
  const [language, setLanguage] = useState("");
  const [tags, setTags] = useState("");
  const [status, setStatus] = useState<ProjectStatus>("ongoing");

  useEffect(() => {
    if (!projectId) return;
    setLoading(true);
    projectApi.get(projectId).then((p) => {
      setProject(p);
      setTitle(p.title);
      setDescription(p.description);
      setAuthor(p.author);
      setLanguage(p.language);
      setTags(p.tags);
      setStatus(p.status as ProjectStatus);
    }).catch((err) => console.error("Failed to load project:", err)).finally(() => setLoading(false));
  }, [projectId]);

  const handleSave = async () => {
    if (!projectId) return;
    setSaving(true);
    setSaved(false);
    try {
      const updated = await projectApi.update(projectId, title.trim(), description.trim(), author.trim(), language.trim(), tags.trim(), status);
      setProject(updated);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (err) {
      console.error("Failed to save project:", err);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex flex-col items-center gap-3">
          <Spinner className="size-5 text-primary" />
          <p className="text-sm text-muted-foreground">加载中...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between px-5 h-11 shrink-0">
        <h1 className="text-sm font-medium text-foreground">书籍详情</h1>
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-6">
        <div className="mx-auto max-w-2xl flex flex-col gap-5">
          <Card>
            <CardHeader>
              <CardTitle>基本信息</CardTitle>
              <CardDescription>书名、作者和简介</CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-4">
              <div className="flex flex-col gap-1.5">
                <Label>书名</Label>
                <Input
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="输入书名..."
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
                  {statusOptions.map((s) => (
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
                  rows={4}
                />
              </div>
              <div className="flex justify-end pt-2">
                <Button
                  onClick={handleSave}
                  disabled={saving || !title.trim()}
                  data-icon="inline-start"
                >
                  {saved ? (
                    <>
                      <Check />
                      已保存
                    </>
                  ) : (
                    <>
                      <Save />
                      {saving ? "保存中..." : "保存"}
                    </>
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>

          {project && (
            <Card>
              <CardHeader>
                <CardTitle>统计</CardTitle>
                <CardDescription>项目创建和更新时间</CardDescription>
              </CardHeader>
              <CardContent className="flex flex-col gap-2 text-sm text-muted-foreground">
                <div className="flex justify-between">
                  <span>创建时间</span>
                  <span>{new Date(project.created_at).toLocaleDateString("zh-CN")}</span>
                </div>
                <div className="flex justify-between">
                  <span>更新时间</span>
                  <span>{new Date(project.updated_at).toLocaleDateString("zh-CN")}</span>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
};

export default ProjectDetailsPage;
