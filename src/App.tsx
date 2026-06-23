import { useEffect } from "react";
import { WorkspaceProvider } from "./context/WorkspaceContext";
import { useWorkspace } from "./hooks/useWorkspace";
import { AppShell } from "./components/Layout/AppShell";

function AppInner() {
  const { toggleCmdk, closeCmdk } = useWorkspace();

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

  return <AppShell />;
}

export default function App() {
  return (
    <WorkspaceProvider>
      <AppInner />
    </WorkspaceProvider>
  );
}
