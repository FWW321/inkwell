import { BrowserRouter, Routes, Route } from "react-router-dom";
import { TooltipProvider } from "@/components/ui/tooltip";
import AppLayout from "@/components/layout/AppLayout";
import ProjectLayout from "@/components/layout/ProjectLayout";
import ProjectsPage from "@/pages/ProjectsPage";
import EditorPage from "@/pages/EditorPage";
import CharactersPage from "@/pages/CharactersPage";
import WorldviewPage from "@/pages/WorldviewPage";
import SettingsPage from "@/pages/SettingsPage";

const App = () => {
  return (
    <TooltipProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="/" element={<ProjectsPage />} />
          </Route>
          <Route path="/project/:projectId" element={<ProjectLayout />}>
            <Route path="write" element={<EditorPage />} />
            <Route path="write/:chapterId" element={<EditorPage />} />
            <Route path="characters" element={<CharactersPage />} />
            <Route path="worldview" element={<WorldviewPage />} />
            <Route path="settings" element={<SettingsPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </TooltipProvider>
  );
};

export default App;
