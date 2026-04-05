import { useParams } from "react-router-dom";
import { Feather } from "lucide-react";
import { AiEditorProvider } from "@/contexts/AiEditorContext";
import Editor from "@/components/editor/Editor";
import { Empty, EmptyHeader, EmptyMedia, EmptyTitle, EmptyDescription } from "@/components/ui/empty";

const EditorPage = () => {
  const { chapterId } = useParams<{ chapterId: string }>();

  if (!chapterId) {
    return (
      <Empty className="h-full">
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <Feather />
          </EmptyMedia>
          <EmptyTitle>选择一个章节开始写作</EmptyTitle>
          <EmptyDescription>
            在左侧创建卷和章节，然后选择一个章节开始编辑
          </EmptyDescription>
        </EmptyHeader>
      </Empty>
    );
  }

  return (
    <AiEditorProvider>
      <div className="h-full">
        <Editor key={chapterId} chapterId={chapterId} />
      </div>
    </AiEditorProvider>
  );
};

export default EditorPage;
