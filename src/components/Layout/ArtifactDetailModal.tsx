import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { useDialog } from "../../hooks/useDialog";
import { parseMarkdown } from "../../lib/commands";
import { MarkdownRenderer } from "../../lib/markdown";
import type { ParsedDocument } from "../../types";
import { GovernancePromptSection } from "./GovernancePromptSection";

export function ArtifactDetailModal() {
  const { state, openDetailArtifact } = useWorkspace();
  const [content, setContent] = useState<string>("");
  const [loadedPath, setLoadedPath] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [doc, setDoc] = useState<ParsedDocument | null>(null);

  const artifact = state.detailArtifact;
  const path = artifact?.path;
  const activeError = loadedPath === path ? error : null;
  const loading = path ? path !== loadedPath && !activeError : false;

  const { dialogRef, handleKeyDown } = useDialog<HTMLDivElement>({
    isOpen: Boolean(artifact && path),
    onClose: () => openDetailArtifact(null),
  });

  useEffect(() => {
    if (!path) return;

    let cancelled = false;
    parseMarkdown(path)
      .then((parsedDoc) => {
        if (cancelled) return;
        setError(null);
        setDoc(parsedDoc);
        setContent(parsedDoc.body);
        setLoadedPath(path);
      })
      .catch((err) => {
        if (cancelled) return;
        setError(typeof err === "string" ? err : "Failed to load artifact content");
        setLoadedPath(path);
      });
    return () => {
      cancelled = true;
    };
  }, [path]);

  if (!artifact || !path) return null;

  const loadedDoc = loadedPath === path ? doc : null;
  const id = loadedDoc?.frontmatter?.id as string | undefined;
  const status = loadedDoc?.frontmatter?.status as string | undefined;
  const isSpec = id?.startsWith("SPEC-") ?? false;
  const isAdr = id?.startsWith("ADR-") ?? false;
  const isAgentProfile = (id?.startsWith("AGENT-") ?? false) && !(id?.startsWith("AGENT-PROP-") ?? false);
  const isGovernanceControlled = isSpec || isAdr || isAgentProfile;
  const showGovernancePrompt = isGovernanceControlled && !!id && !!status
    && ((isSpec && status === "backlog") || (isAdr && status === "proposed") || (isAgentProfile && status === "proposed"));

  return (
    <div
      role="presentation"
      onKeyDown={handleKeyDown}
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
      onClick={() => openDetailArtifact(null)}
    >
      {/* eslint-disable-next-line jsx-a11y/click-events-have-key-events, jsx-a11y/no-noninteractive-element-interactions */}
      <div
        ref={dialogRef}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-label={`Detail for ${artifact.title}`}
        style={{
          background: "var(--bg-secondary)",
          border: "1px solid var(--border-primary)",
          borderRadius: 16,
          width: "100%",
          maxWidth: "var(--page-reading)",
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
            <h2 style={{ margin: 0, fontSize: "var(--text-lg)", fontWeight: 700, color: "var(--text-primary)" }}>
              {artifact.title}
            </h2>
            <span style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", fontFamily: "var(--font-mono)" }}>
              {artifact.path}
            </span>
          </div>
          <button
            type="button"
            onClick={() => openDetailArtifact(null)}
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
          ) : activeError ? (
            <div style={{ color: "#e0584a", padding: 20, textAlign: "center" }}>{activeError}</div>
          ) : (
            <>
              <MarkdownRenderer content={content} />
              <GovernancePromptSection
                id={id ?? ""}
                title={artifact.title}
                path={artifact.path}
                showGovernancePrompt={showGovernancePrompt}
                status={status}
              />
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
              type="button"
              onClick={() => openDetailArtifact(null)}
              style={{
                background: "rgba(255,255,255,0.06)",
                border: "1px solid rgba(255,255,255,0.1)",
                borderRadius: 8,
                padding: "6px 14px",
                fontSize: "var(--text-sm)",
                color: "#fff",
                cursor: "pointer",
                fontWeight: 600,
              }}
              onMouseEnter={(e) => (e.currentTarget.style.background = "rgba(255,255,255,0.1)")}
              onMouseLeave={(e) => (e.currentTarget.style.background = "rgba(255,255,255,0.06)")}
            >
              Close
            </button>
          </div>
          <span style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
            Read-only · approval and lifecycle status actions are unavailable in the app
          </span>
        </div>
      </div>
    </div>
  );
}
