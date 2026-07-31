import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useWorkspace } from "../../hooks/useWorkspace";
import * as commands from "../../lib/commands";

export function WindowCloseConfirmModal() {
  const { state, dispatch } = useWorkspace();
  const [status, setStatus] = useState<"idle" | "closing" | "failed">("idle");
  const [failures, setFailures] = useState<string[]>([]);
  const modalRef = useRef<HTMLDivElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const [openSessions] = useState(() => state.sessions);
  const activeSessions = openSessions.filter((session) => session.status === "running");

  const dismiss = () => {
    if (status === "closing") return;
    dispatch({ type: "SET_WINDOW_CLOSE_CONFIRM", show: false });
  };

  useEffect(() => {
    previousFocus.current = document.activeElement as HTMLElement;
    modalRef.current?.querySelector<HTMLElement>("[data-safe-action]")?.focus();
    return () => previousFocus.current?.focus();
  }, []);

  const closeWindow = async (force = false) => {
    setStatus("closing");
    setFailures([]);
    const errors: string[] = [];

    if (!force) {
      try {
        await commands.stopWatcher();
      } catch (error) {
        errors.push(`File watcher: ${String(error)}`);
      }
      const results = await Promise.allSettled(
        activeSessions.map((session) => commands.sessionKill(session.id)),
      );
      results.forEach((result, index) => {
        if (result.status === "rejected") {
          errors.push(`${activeSessions[index].label}: ${String(result.reason)}`);
        }
      });
    }

    if (errors.length > 0) {
      setFailures(errors);
      setStatus("failed");
      return;
    }

    try {
      await getCurrentWindow().destroy();
    } catch (error) {
      setFailures([`Window: ${String(error)}`]);
      setStatus("failed");
    }
  };

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      dismiss();
      return;
    }
    if (event.key !== "Tab" || !modalRef.current) return;
    const focusable = modalRef.current.querySelectorAll<HTMLElement>(
      'button:not(:disabled), [href], [tabindex]:not([tabindex="-1"])',
    );
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      last.focus();
      event.preventDefault();
    } else if (!event.shiftKey && document.activeElement === last) {
      first.focus();
      event.preventDefault();
    }
  };

  return (
    <div
      role="presentation"
      onKeyDown={handleKeyDown}
      onClick={(event) => {
        if (event.target === event.currentTarget) dismiss();
      }}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 13000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 24,
        background: "rgba(4,3,6,.76)",
        backdropFilter: "blur(6px)",
      }}
    >
      <div
        ref={modalRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="window-close-title"
        aria-describedby="window-close-description"
        style={{
          width: "min(440px, 100%)",
          padding: 22,
          borderRadius: 13,
          border: "1px solid #3a3045",
          background: "#15111b",
          boxShadow: "0 24px 80px rgba(0,0,0,.58)",
          color: "var(--text-primary)",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 12 }}>
          <div
            aria-hidden="true"
            style={{
              width: 38,
              height: 38,
              borderRadius: 10,
              display: "grid",
              placeItems: "center",
              background: "rgba(224,162,58,.10)",
              border: "1px solid rgba(224,162,58,.24)",
              color: "#e0a23a",
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 21 }}>warning</i>
          </div>
          <div>
            <h2 id="window-close-title" style={{ margin: 0, fontSize: "var(--text-xl)", fontWeight: 750 }}>
              Close LMBrain?
            </h2>
            <div style={{ marginTop: 3, fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
              {activeSessions.length > 0
                ? "Active agent work will be stopped"
                : "Open session tabs will be closed"}
            </div>
          </div>
        </div>

        <p
          id="window-close-description"
          style={{ margin: "0 0 17px", fontSize: "var(--text-md)", lineHeight: 1.55, color: "var(--text-secondary)" }}
        >
          {openSessions.length === 1
            ? `The session “${openSessions[0].label}” is still open.`
            : `${openSessions.length} agent sessions are still open.`}{" "}
          {activeSessions.length > 0
            ? `Closing the application will stop ${activeSessions.length === 1 ? "the running process" : `${activeSessions.length} running processes`}.`
            : "Closing the application will discard the open session tabs."}
        </p>

        {status === "closing" && (
          <div role="status" style={{ marginBottom: 16, color: "#bcaef6", fontSize: "var(--text-sm)" }}>
            Stopping active sessions…
          </div>
        )}

        {status === "failed" && (
          <div
            role="alert"
            style={{
              marginBottom: 16,
              padding: "10px 12px",
              borderRadius: 8,
              background: "rgba(224,88,74,.08)",
              border: "1px solid rgba(224,88,74,.24)",
              color: "#f08075",
              fontSize: "var(--text-xs)",
            }}
          >
            Some processes could not be stopped: {failures.join("; ")}
          </div>
        )}

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 9 }}>
          <button
            type="button"
            data-safe-action
            disabled={status === "closing"}
            onClick={dismiss}
            style={{
              border: "1px solid #332d3e",
              borderRadius: 8,
              background: "#1b1721",
              color: "var(--text-secondary)",
              padding: "8px 13px",
              fontSize: "var(--text-sm)",
              fontWeight: 650,
              cursor: status === "closing" ? "wait" : "pointer",
            }}
          >
            Keep app open
          </button>
          <button
            type="button"
            disabled={status === "closing"}
            onClick={() => void closeWindow(status === "failed")}
            style={{
              border: "none",
              borderRadius: 8,
              background: status === "failed"
                ? "linear-gradient(135deg,#e0584a,#c44437)"
                : "linear-gradient(135deg,#7c6cf6,#6553df)",
              color: "#fff",
              padding: "8px 13px",
              fontSize: "var(--text-sm)",
              fontWeight: 750,
              cursor: status === "closing" ? "wait" : "pointer",
              opacity: status === "closing" ? 0.65 : 1,
            }}
          >
            {status === "failed" ? "Close anyway" : "Close LMBrain"}
          </button>
        </div>
      </div>
    </div>
  );
}
