import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { WorkspaceProvider } from "./context/WorkspaceContext";
import { useWorkspace } from "./hooks/useWorkspace";
import { AppShell } from "./components/Layout/AppShell";
import { resolveOpenSessions, routeWindowCloseRequest } from "./lib/windowClose";
import * as commands from "./lib/commands";

function AppInner() {
  const { toggleCmdk, closeCmdk, state, setSessions, setShowWindowCloseConfirm } = useWorkspace();
  const sessionsRef = useRef(state.sessions);
  const closeRequestInFlight = useRef(false);

  useEffect(() => {
    sessionsRef.current = state.sessions;
  }, [state.sessions]);

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

  // Tauri prevents native close while a JS close listener exists. Route every
  // request explicitly: destroy immediately when safe, otherwise show our modal.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    async function setupCloseHandler() {
      try {
        const appWindow = getCurrentWindow();
        const removeListener = await appWindow.onCloseRequested(async (event) => {
          // @tauri-apps/api destroys the window after this handler resolves
          // unless preventDefault is set. Claim the request synchronously, then
          // make the close/confirm decision ourselves.
          event.preventDefault();
          if (closeRequestInFlight.current) return;
          closeRequestInFlight.current = true;
          try {
            const openSessions = await resolveOpenSessions({
              listSessions: commands.sessionList,
              fallbackSessions: sessionsRef.current,
            });
            if (openSessions.length > 0) {
              setSessions(openSessions);
            }
            await routeWindowCloseRequest({
              openSessionCount: openSessions.length,
              destroy: () => appWindow.destroy(),
              showConfirmation: () => setShowWindowCloseConfirm(true),
            });
          } finally {
            closeRequestInFlight.current = false;
          }
        });
        if (disposed) {
          removeListener();
        } else {
          unlisten = removeListener;
        }
      } catch {
        // Fallback for non-Tauri browser environments
      }
    }
    void setupCloseHandler();
    return () => {
      disposed = true;
      if (unlisten) unlisten();
    };
  }, [setSessions, setShowWindowCloseConfirm]);

  return <AppShell />;
}

export default function App() {
  return (
    <WorkspaceProvider>
      <AppInner />
    </WorkspaceProvider>
  );
}
