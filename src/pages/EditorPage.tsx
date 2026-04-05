import { useParams } from "react-router-dom";
import { Feather } from "lucide-react";
import Editor from "@/components/editor/Editor";

const EditorPage = () => {
  const { chapterId } = useParams<{ chapterId: string }>();

  if (!chapterId) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-5">
        <div className="flex h-20 w-20 items-center justify-center rounded-2xl bg-primary/10">
          <Feather className="h-10 w-10 text-primary/60" />
        </div>
        <div className="text-center">
          <p className="text-lg font-semibold text-foreground">
            选择一个章节开始写作
          </p>
          <p className="mt-1 text-sm text-muted-foreground">
            在左侧创建卷和章节，然后选择一个章节开始编辑
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full">
      <Editor key={chapterId} chapterId={chapterId} />
    </div>
  );
};

export default EditorPage;
