import { useEffect, useMemo, useState } from "react";
import * as commands from "../../lib/commands";
import { parseUnifiedDiff } from "../../lib/gitDiff";
import type { GitFile, GitFileDiff } from "../../types";
import { ModalCloseButton } from "../Layout/ModalCloseButton";
import { useDialog } from "../../hooks/useDialog";
import "./RepositoryView.css";

interface GitDiffModalProps {
  file: GitFile;
  /** Worktree name when the file belongs to a linked worktree; main tree otherwise. */
  worktree?: string;
  onClose: () => void;
}

const PREVIEW_LINE_LIMIT = 5000;

function message(value: unknown): string {
  if (typeof value === "string") return value;
  if (value instanceof Error && value.message) return value.message;
  return "Git could not load this diff";
}

export function GitDiffModal({ file, worktree, onClose }: GitDiffModalProps) {
  const [result, setResult] = useState<GitFileDiff | null>(null);
  const [error, setError] = useState<string | null>(null);
  const { dialogRef, handleKeyDown } = useDialog<HTMLDivElement>({ isOpen: true, onClose });

  useEffect(() => {
    let active = true;
    commands
      .getGitFileDiff(file.path, file.diff_target, worktree)
      .then((nextResult) => {
        if (active) setResult(nextResult);
      })
      .catch((reason: unknown) => {
        if (active) setError(message(reason));
      });
    return () => {
      active = false;
    };
  }, [file.diff_target, file.path, worktree]);

  const lines = useMemo(() => parseUnifiedDiff(result?.diff ?? ""), [result?.diff]);
  const visibleLines = lines.slice(0, PREVIEW_LINE_LIMIT);
  const linesTruncated = lines.length > PREVIEW_LINE_LIMIT;

  return (
    <div
      className="repository-diff-overlay"
      role="presentation"
      onKeyDown={handleKeyDown}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions */}
      <div
        ref={dialogRef}
        className="repository-diff-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="repository-diff-title"
        tabIndex={-1}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="repository-diff-header">
          <div className="repository-diff-header-copy">
            <div style={{ display: "flex", alignItems: "center", gap: 8, minWidth: 0 }}>
              <h2 id="repository-diff-title" className="repository-ellipsis" title={file.path} style={{ margin: 0, fontSize: "var(--text-lg)", color: "var(--text-primary)" }}>
                {file.path}
              </h2>
              <span style={{ flexShrink: 0, padding: "2px 6px", borderRadius: 5, background: "rgba(124,108,246,.14)", color: "#b9adff", fontSize: "var(--text-2xs)", fontWeight: 700, textTransform: "uppercase" }}>
                {file.status}
              </span>
            </div>
            <div style={{ marginTop: 3, color: "var(--text-tertiary)", fontSize: "var(--text-xs)" }}>
              {file.diff_target === "staged" ? "Index changes" : file.diff_target === "untracked" ? "Untracked file" : "Working tree changes"}
            </div>
          </div>
          <ModalCloseButton label="Close diff" onClick={onClose} />
        </div>

        {(result?.truncated || linesTruncated) && (
          <div role="status" style={{ padding: "8px 18px", borderBottom: "1px solid rgba(245,158,11,.22)", background: "rgba(245,158,11,.08)", color: "#f6c86b", fontSize: "var(--text-xs)" }}>
            {result?.truncated
              ? "Diff truncated at 512 KiB. Use a dedicated source-control tool for the complete patch."
              : `Preview limited to the first ${PREVIEW_LINE_LIMIT.toLocaleString()} lines. Use a dedicated source-control tool for the complete patch.`}
          </div>
        )}

        <div className="repository-diff-body">
          {!result && !error ? (
            <div style={{ padding: 40, textAlign: "center", color: "var(--text-tertiary)", fontFamily: "var(--font-sans)" }}>Loading diff...</div>
          ) : error ? (
            <div role="alert" style={{ padding: 40, textAlign: "center", color: "#ef8a80", fontFamily: "var(--font-sans)" }}>{error}</div>
          ) : result?.binary ? (
            <div style={{ padding: 40, textAlign: "center", color: "var(--text-secondary)", fontFamily: "var(--font-sans)" }}>Binary file changed. A textual preview is not available.</div>
          ) : !result?.diff.trim() ? (
            <div style={{ padding: 40, textAlign: "center", color: "var(--text-tertiary)", fontFamily: "var(--font-sans)" }}>No textual changes are available for this file.</div>
          ) : (
            visibleLines.map((line, index) => (
              <div className={`repository-diff-line repository-diff-line--${line.kind}`} key={`${index}-${line.oldLine ?? ""}-${line.newLine ?? ""}`}>
                <span className="repository-diff-number">{line.oldLine ?? ""}</span>
                <span className="repository-diff-number">{line.newLine ?? ""}</span>
                <span className="repository-diff-code">{line.content || " "}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
