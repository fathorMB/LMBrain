import { useEffect, useState, useRef } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { parseMarkdown, setArtifactStatus } from "../../lib/commands";
import { MarkdownRenderer } from "../../lib/markdown";
import type { ParsedDocument } from "../../types";

function getTargetStatuses(id: string): { approve: string; reject: string; rejectLabel?: string } | null {
  if (id.startsWith("SPEC-")) {
    return { approve: "ready", reject: "rejected" };
  }
  if (id.startsWith("ADR-")) {
    return { approve: "accepted", reject: "rejected" };
  }
  if (id.startsWith("AGENT-PROP-")) {
    return { approve: "approved", reject: "rejected" };
  }
  if (id.startsWith("AGENT-")) {
    return { approve: "active", reject: "inactive", rejectLabel: "Deactivate" };
  }
  if (id.startsWith("MCP-PROP-")) {
    return { approve: "approved", reject: "rejected" };
  }
  return null;
}

function generateRejectedPrompt(path: string, id: string): string {
  return `Please revise the rejected artifact: ${path} (${id})
This artifact has been rejected by the operator.

Instructions:
1. Review the artifact structure and contents.
2. Address the reasons for rejection or make the necessary updates to improve it.
3. Once the revisions are complete, set its status back to "proposed" so it can be reviewed again.
4. Do not make any unrelated changes to other files.`;
}

export function ArtifactDetailModal() {
  const { state, dispatch, loadAllData } = useWorkspace();
  const [content, setContent] = useState<string>("");
  const [loadedPath, setLoadedPath] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const modalRef = useRef<HTMLDivElement | null>(null);

  const [doc, setDoc] = useState<ParsedDocument | null>(null);
  const [promptCopied, setPromptCopied] = useState(false);
  const [confirmAction, setConfirmAction] = useState<"approve" | "reject" | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const [prevPath, setPrevPath] = useState<string>("");

  const artifact = state.detailArtifact;
  const path = artifact?.path;
  const loading = path ? path !== loadedPath && !error : false;

  if (path !== prevPath) {
    setPrevPath(path || "");
    setConfirmAction(null);
    setSubmitError(null);
    setSubmitting(false);
  }

  // Restore focus when the modal unmounts
  useEffect(() => {
    const prev = document.activeElement as HTMLElement;
    return () => {
      if (prev) {
        prev.focus();
      }
    };
  }, []);

  // Fetch the artifact content on mount/path change
  useEffect(() => {
    if (!path) return;

    parseMarkdown(path)
      .then((parsedDoc) => {
        setDoc(parsedDoc);
        setContent(parsedDoc.body);
        setLoadedPath(path);
        if (modalRef.current) {
          modalRef.current.focus();
        }
      })
      .catch((err) => {
        setError(typeof err === "string" ? err : "Failed to load artifact content");
        setLoadedPath(path);
      });
  }, [path]);

  // Handle ESC and Tab trap
  useEffect(() => {
    if (!path) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: null });
      }
      if (e.key === "Tab" && modalRef.current) {
        const focusableElements = modalRef.current.querySelectorAll(
          'a[href], button, textarea, input, select, [tabindex]:not([tabindex="-1"])'
        );
        if (focusableElements.length === 0) {
          e.preventDefault();
          return;
        }
        const firstElement = focusableElements[0] as HTMLElement;
        const lastElement = focusableElements[focusableElements.length - 1] as HTMLElement;

        if (e.shiftKey) {
          if (document.activeElement === firstElement) {
            lastElement.focus();
            e.preventDefault();
          }
        } else {
          if (document.activeElement === lastElement) {
            firstElement.focus();
            e.preventDefault();
          }
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [path, dispatch]);

  if (!artifact || !path) return null;

  const id = doc?.frontmatter?.id as string | undefined;
  const status = doc?.frontmatter?.status as string | undefined;
  const eligibleTransitions = id && status === "proposed" ? getTargetStatuses(id) : null;

  const handleAction = async (targetStatus: string) => {
    if (!artifact) return;
    setSubmitting(true);
    setSubmitError(null);
    try {
      const newPath = await setArtifactStatus(artifact.path, targetStatus);
      await loadAllData();
      setConfirmAction(null);
      dispatch({
        type: "SET_DETAIL_ARTIFACT",
        artifact: { title: artifact.title, path: newPath },
      });
    } catch (err) {
      console.error(err);
      setSubmitError(typeof err === "string" ? err : "Failed to update artifact status");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        background: "rgba(9, 7, 12, 0.8)",
        backdropFilter: "blur(4px)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
        padding: 24,
      }}
      onClick={() => dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: null })}
    >
      <div
        ref={modalRef}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-label={`Detail for ${artifact.title}`}
        style={{
          background: "var(--bg-secondary)",
          border: "1px solid var(--border-primary)",
          borderRadius: 16,
          width: "100%",
          maxWidth: 800,
          maxHeight: "90vh",
          display: "flex",
          flexDirection: "column",
          outline: "none",
          boxShadow: "0 20px 25px -5px rgba(0,0,0,0.5), 0 10px 10px -5px rgba(0,0,0,0.5)",
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "16px 24px",
            borderBottom: "1px solid var(--border-primary)",
          }}
        >
          <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
            <h2 style={{ margin: 0, fontSize: 16, fontWeight: 700, color: "var(--text-primary)" }}>
              {artifact.title}
            </h2>
            <span style={{ fontSize: 11.5, color: "var(--text-tertiary)", fontFamily: "var(--font-mono)" }}>
              {artifact.path}
            </span>
          </div>
          <button
            onClick={() => dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: null })}
            aria-label="Close modal"
            style={{
              background: "transparent",
              border: "none",
              color: "var(--text-tertiary)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              padding: 4,
              borderRadius: 6,
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.color = "#fff";
              e.currentTarget.style.background = "rgba(255,255,255,0.06)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.color = "var(--text-tertiary)";
              e.currentTarget.style.background = "transparent";
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 20 }}>
              close
            </i>
          </button>
        </div>

        {/* Content body */}
        <div style={{ flex: 1, overflowY: "auto", padding: "24px 32px", minHeight: 0 }}>
          {loading ? (
            <div style={{ textAlign: "center", padding: 40, color: "var(--text-tertiary)" }}>
              Loading content...
            </div>
          ) : error ? (
            <div style={{ color: "#e0584a", padding: 20, textAlign: "center" }}>{error}</div>
          ) : (
            <>
              <MarkdownRenderer content={content} />
              {status === "rejected" && id && (
                <div
                  style={{
                    marginTop: 24,
                    padding: 16,
                    background: "rgba(224, 88, 74, 0.08)",
                    border: "1px solid rgba(224, 88, 74, 0.2)",
                    borderRadius: 10,
                  }}
                >
                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
                    <i className="material-symbols-outlined" style={{ color: "#e0584a", fontSize: 20 }}>
                      info
                    </i>
                    <span style={{ fontSize: 13.5, fontWeight: 600, color: "#fff" }}>
                      Artifact Rejected
                    </span>
                  </div>
                  <p style={{ fontSize: 12.5, color: "var(--text-secondary)", margin: "0 0 12px" }}>
                    This proposal was rejected. Copy the corrective prompt below to have an agent revise the file:
                  </p>
                  <div style={{ position: "relative" }}>
                    <textarea
                      readOnly
                      value={generateRejectedPrompt(artifact.path, id)}
                      style={{
                        width: "100%",
                        height: 120,
                        background: "var(--bg-primary)",
                        border: "1px solid var(--border-primary)",
                        borderRadius: 6,
                        padding: "8px 12px",
                        fontFamily: "var(--font-mono)",
                        fontSize: 11.5,
                        color: "var(--text-secondary)",
                        resize: "none",
                        outline: "none",
                      }}
                    />
                    <button
                      onClick={() => {
                        navigator.clipboard.writeText(generateRejectedPrompt(artifact.path, id));
                        setPromptCopied(true);
                        setTimeout(() => setPromptCopied(false), 2000);
                      }}
                      style={{
                        position: "absolute",
                        right: 8,
                        bottom: 12,
                        background: "rgba(255,255,255,0.06)",
                        border: "1px solid rgba(255,255,255,0.1)",
                        borderRadius: 6,
                        padding: "4px 10px",
                        fontSize: 11.5,
                        color: "#fff",
                        cursor: "pointer",
                        fontWeight: 600,
                      }}
                    >
                      {promptCopied ? "Copied!" : "Copy prompt"}
                    </button>
                  </div>
                </div>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div
          style={{
            padding: "12px 24px",
            borderTop: "1px solid var(--border-primary)",
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            background: "var(--bg-tertiary)",
            borderBottomLeftRadius: 16,
            borderBottomRightRadius: 16,
          }}
        >
          <div>
            <button
              onClick={() => dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: null })}
              style={{
                background: "rgba(255,255,255,0.06)",
                border: "1px solid rgba(255,255,255,0.1)",
                borderRadius: 8,
                padding: "6px 14px",
                fontSize: 12.5,
                color: "#fff",
                cursor: "pointer",
                fontWeight: 600,
              }}
              onMouseEnter={(e) => (e.currentTarget.style.background = "rgba(255,255,255,0.1)")}
              onMouseLeave={(e) => (e.currentTarget.style.background = "rgba(255,255,255,0.06)")}
            >
              Close
            </button>
            {submitError && (
              <span style={{ color: "#e0584a", fontSize: 12.5, marginLeft: 16 }}>
                {submitError}
              </span>
            )}
          </div>

          {eligibleTransitions && (
            <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
              {confirmAction === null ? (
                <>
                  <button
                    disabled={submitting}
                    onClick={() => { console.log("reject button clicked!"); setConfirmAction("reject"); }}
                    style={{
                      background: "rgba(224, 88, 74, 0.15)",
                      border: "1px solid rgba(224, 88, 74, 0.4)",
                      borderRadius: 8,
                      padding: "6px 14px",
                      fontSize: 12.5,
                      color: "#e0584a",
                      cursor: "pointer",
                      fontWeight: 600,
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.background = "rgba(224, 88, 74, 0.25)")}
                    onMouseLeave={(e) => (e.currentTarget.style.background = "rgba(224, 88, 74, 0.15)")}
                  >
                    {eligibleTransitions.rejectLabel || "Reject"}
                  </button>
                  <button
                    disabled={submitting}
                    onClick={() => setConfirmAction("approve")}
                    style={{
                      background: "rgba(70, 176, 125, 0.15)",
                      border: "1px solid rgba(70, 176, 125, 0.4)",
                      borderRadius: 8,
                      padding: "6px 14px",
                      fontSize: 12.5,
                      color: "#46b07d",
                      cursor: "pointer",
                      fontWeight: 600,
                    }}
                    onMouseEnter={(e) => (e.currentTarget.style.background = "rgba(70, 176, 125, 0.25)")}
                    onMouseLeave={(e) => (e.currentTarget.style.background = "rgba(70, 176, 125, 0.15)")}
                  >
                    Approve
                  </button>
                </>
              ) : (
                <>
                  <span style={{ fontSize: 12.5, color: "var(--text-secondary)", marginRight: 4 }}>
                    Confirm {confirmAction === "approve" ? "Approval" : "Rejection"}?
                  </span>
                  <button
                    disabled={submitting}
                    onClick={() => setConfirmAction(null)}
                    style={{
                      background: "rgba(255,255,255,0.06)",
                      border: "1px solid rgba(255,255,255,0.1)",
                      borderRadius: 8,
                      padding: "6px 12px",
                      fontSize: 12.5,
                      color: "#fff",
                      cursor: "pointer",
                      fontWeight: 600,
                    }}
                  >
                    Cancel
                  </button>
                  <button
                    disabled={submitting}
                    onClick={() =>
                      handleAction(
                        confirmAction === "approve"
                          ? eligibleTransitions.approve
                          : eligibleTransitions.reject
                      )
                    }
                    style={{
                      background: confirmAction === "approve" ? "#46b07d" : "#e0584a",
                      border: "none",
                      borderRadius: 8,
                      padding: "6px 14px",
                      fontSize: 12.5,
                      color: "#fff",
                      cursor: "pointer",
                      fontWeight: 600,
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.background =
                        confirmAction === "approve" ? "#3da06e" : "#d04d40";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.background =
                        confirmAction === "approve" ? "#46b07d" : "#e0584a";
                    }}
                  >
                    {submitting ? "Processing..." : "Yes, Confirm"}
                  </button>
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
