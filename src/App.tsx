import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { confirm } from "@tauri-apps/plugin-dialog";
import { WorkspaceProvider } from "./context/WorkspaceContext";
import { useWorkspace } from "./hooks/useWorkspace";
import { AppShell } from "./components/Layout/AppShell";

function AppInner() {
  const { toggleCmdk, closeCmdk, state } = useWorkspace();

  // Global keyboard shortcuts
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const k = e.key.toLowerCase();
      if ((e.metaKey || e.ctrlKey) && k === "k") {
        e.preventDefault();
        toggleCmdk();
      } else if (e.key === "Escape") {
        closeCmdk();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleCmdk, closeCmdk]);

  // Intercept window close when active sessions are open
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    async function setupCloseHandler() {
      try {
        const appWindow = getCurrentWindow();
        unlisten = await appWindow.onCloseRequested(async (event) => {
          if (state.sessions && state.sessions.length > 0) {
            const confirmed = await confirm(
              "Active agent sessions are open. Are you sure you want to exit LMBrain?",
              { title: "Close LMBrain?", kind: "warning" }
            );
            if (!confirmed) {
              event.preventDefault();
            }
          }
        });
      } catch {
        // Fallback for non-Tauri browser environments
      }
    }
    setupCloseHandler();
    return () => {
      if (unlisten) unlisten();
    };
  }, [state.sessions]);

  return <AppShell />;
}

export default function App() {
  return (
    <WorkspaceProvider>
      <AppInner />
    </WorkspaceProvider>
  );
}
