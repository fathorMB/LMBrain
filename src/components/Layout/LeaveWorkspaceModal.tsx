import { useEffect, useRef, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import * as commands from "../../lib/commands";

export function LeaveWorkspaceModal() {
  const { state, goToPicker, cancelLeaveWorkspace } = useWorkspace();
  const [status, setStatus] = useState<"idle" | "leaving" | "failed">("idle");
  const [failures, setFailures] = useState<string[]>([]);
  
  const modalRef = useRef<HTMLDivElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);

  useEffect(() => {
    // Record current focus to restore on dismissal
    previousFocus.current = document.activeElement as HTMLElement;

    // Focus safe action button
    const safeBtn = modalRef.current?.querySelector("[data-safe-action]") as HTMLElement;
    safeBtn?.focus();

    return () => {
      // Restore focus to triggering control
      previousFocus.current?.focus();
    };
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      cancelLeaveWorkspace();
      return;
    }
    if (e.key === "Tab" && modalRef.current) {
      const focusables = modalRef.current.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      if (focusables.length === 0) return;
      const first = focusables[0];
      const last = focusables[focusables.length - 1];

      if (e.shiftKey) {
        if (document.activeElement === first) {
          last.focus();
          e.preventDefault();
        }
      } else {
        if (document.activeElement === last) {
          first.focus();
          e.preventDefault();
        }
      }
    }
  };

  const handleLeave = async () => {
    setStatus("leaving");
    setFailures([]);
    
    const errors: string[] = [];

    // 1. Stop workspace watcher
    try {
      await commands.stopWatcher();
    } catch (err) {
      errors.push(`File Watcher: ${err}`);
    }

    // 2. Kill all active agent sessions
    for (const session of state.sessions) {
      try {
        await commands.sessionKill(session.id);
      } catch (err) {
        errors.push(`Session "${session.label}": ${err}`);
      }
    }

    if (errors.length > 0) {
      setFailures(errors);
      setStatus("failed");
    } else {
      await goToPicker();
    }
  };

  const handleForceLeave = async () => {
    // Destructive skip: directly trigger context transition
    await goToPicker();
  };

  if (!state.showExitConfirm) {
    return null;
  }

  return (
    <div
      role="presentation"
      onKeyDown={handleKeyDown}
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          cancelLeaveWorkspace();
        }
      }}
      style={{
        position: "fixed",
        inset: 0,
        zIndex: 12000,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 24,
        background: "rgba(4,3,6,.72)",
        backdropFilter: "blur(5px)",
      }}
    >
      <div
        ref={modalRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="leave-workspace-title"
        style={{
          width: "min(460px, 100%)",
          padding: 20,
          borderRadius: 12,
          border: "1px solid #332d3e",
          background: "#15111b",
          boxShadow: "0 20px 70px rgba(0,0,0,.5)",
          color: "var(--text-primary)",
        }}
      >
        <h2
          id="leave-workspace-title"
          style={{
            margin: "0 0 10px",
            fontSize: 17,
            fontWeight: 700,
            color: "var(--text-primary)",
          }}
        >
          Leave this workspace?
        </h2>
        
        <p
          style={{
            margin: "0 0 16px",
            color: "var(--text-secondary)",
            fontSize: 13,
            lineHeight: 1.5,
          }}
        >
          You&apos;ll return to the main menu. Any running agent sessions for this workspace will be stopped.
        </p>

        {status === "leaving" && (
          <div
            style={{
              padding: "10px 12px",
              borderRadius: 7,
              background: "rgba(106,79,240,.08)",
              color: "#bcaef6",
              fontSize: 12,
              marginBottom: 16,
              display: "flex",
              alignItems: "center",
              gap: 8,
            }}
          >
            <span
              style={{
                width: 12,
                height: 12,
                borderRadius: "50%",
                border: "2px solid #9384f8",
                borderTopColor: "transparent",
                animation: "spin 0.8s linear infinite",
                display: "inline-block",
              }}
            />
            Stopping active watchers and agent processes...
            <style>{`
              @keyframes spin {
                to { transform: rotate(360deg); }
              }
            `}</style>
          </div>
        )}

        {status === "failed" && (
          <div
            style={{
              padding: "12px",
              borderRadius: 7,
              background: "rgba(224,88,74,.08)",
              border: "1px solid rgba(224,88,74,.25)",
              color: "#f87171",
              fontSize: 12,
              marginBottom: 16,
            }}
          >
            <div style={{ fontWeight: 700, marginBottom: 6 }}>
              Warning: Some processes failed to stop:
            </div>
            <ul style={{ margin: 0, paddingLeft: 18, lineHeight: 1.45 }}>
              {failures.map((f, i) => (
                <li key={i}>{f}</li>
              ))}
            </ul>
          </div>
        )}

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 12 }}>
          {status === "failed" ? (
            <>
              <button
                type="button"
                onClick={handleLeave}
                style={{
                  border: "1px solid #302a39",
                  borderRadius: 7,
                  background: "#19151f",
                  color: "var(--text-secondary)",
                  padding: "7px 11px",
                  fontSize: 11.5,
                  fontWeight: 600,
                  cursor: "pointer",
                }}
              >
                Retry
              </button>
              <button
                type="button"
                onClick={handleForceLeave}
                style={{
                  border: "none",
                  borderRadius: 7,
                  background: "linear-gradient(135deg,#e0584a,#c0392b)",
                  color: "#fff",
                  padding: "7px 11px",
                  fontSize: 11.5,
                  fontWeight: 700,
                  cursor: "pointer",
                }}
              >
                Force Return
              </button>
              <button
                type="button"
                onClick={cancelLeaveWorkspace}
                style={{
                  border: "1px solid #302a39",
                  borderRadius: 7,
                  background: "#19151f",
                  color: "var(--text-secondary)",
                  padding: "7px 11px",
                  fontSize: 11.5,
                  fontWeight: 600,
                  cursor: "pointer",
                }}
              >
                Cancel
              </button>
            </>
          ) : (
            <>
              <button
                type="button"
                data-safe-action
                disabled={status === "leaving"}
                onClick={cancelLeaveWorkspace}
                style={{
                  border: "1px solid #302a39",
                  borderRadius: 7,
                  background: "#19151f",
                  color: "var(--text-secondary)",
                  padding: "7px 11px",
                  fontSize: 11.5,
                  fontWeight: 600,
                  cursor: status === "leaving" ? "not-allowed" : "pointer",
                  opacity: status === "leaving" ? 0.6 : 1,
                }}
              >
                Stay in workspace
              </button>
              <button
                type="button"
                disabled={status === "leaving"}
                onClick={handleLeave}
                style={{
                  border: "none",
                  borderRadius: 7,
                  background: "linear-gradient(135deg,#e0584a,#c0392b)",
                  color: "#fff",
                  padding: "7px 11px",
                  fontSize: 11.5,
                  fontWeight: 700,
                  cursor: status === "leaving" ? "not-allowed" : "pointer",
                  opacity: status === "leaving" ? 0.6 : 1,
                }}
              >
                Leave workspace
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
