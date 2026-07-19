import { useEffect, useRef } from "react";
import type { GitHubWorkflowRun } from "../../types";
import { getWorkflowRunStatusStyle } from "../../lib/workflowRunStatus";
import "./RepositoryView.css";

interface WorkflowRunModalProps {
  run: GitHubWorkflowRun;
  onClose: () => void;
}

const EMPTY = "—";

function formatTimestamp(value: string | null | undefined): string {
  if (!value) return EMPTY;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? EMPTY : date.toLocaleString();
}

function text(value: string | null | undefined): string {
  return value && value.trim() ? value : EMPTY;
}

function Field({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="repository-run-field">
      <div style={{ fontSize: 10, textTransform: "uppercase", letterSpacing: ".06em", color: "var(--text-muted)", marginBottom: 4 }}>
        {label}
      </div>
      <div
        className="repository-ellipsis"
        title={value}
        style={{ fontSize: 13, fontWeight: 650, color: "var(--text-secondary)", fontFamily: mono ? "var(--font-mono)" : undefined }}
      >
        {value}
      </div>
    </div>
  );
}

export function WorkflowRunModal({ run, onClose }: WorkflowRunModalProps) {
  const modalRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", closeOnEscape);
    modalRef.current?.focus();
    return () => {
      window.removeEventListener("keydown", closeOnEscape);
      previousFocus?.focus();
    };
  }, [onClose]);

  const s = getWorkflowRunStatusStyle(run.status, run.conclusion);

  return (
    <div className="repository-diff-overlay" onMouseDown={onClose}>
      <div
        ref={modalRef}
        className="repository-diff-modal repository-run-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="repository-run-title"
        tabIndex={-1}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="repository-diff-header">
          <div className="repository-diff-header-copy">
            <div style={{ display: "flex", alignItems: "center", gap: 8, minWidth: 0 }}>
              <h2 id="repository-run-title" className="repository-ellipsis" title={run.name} style={{ margin: 0, fontSize: 15, color: "var(--text-primary)" }}>
                {text(run.name)}
                {run.run_number > 0 && (
                  <span style={{ color: "var(--text-tertiary)", fontWeight: 500 }}> #{run.run_number}</span>
                )}
                {run.run_attempt > 1 && (
                  <span style={{ color: "var(--text-tertiary)", fontWeight: 500 }}> · attempt {run.run_attempt}</span>
                )}
              </h2>
              <span
                style={{
                  flexShrink: 0,
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 4,
                  padding: "2px 8px",
                  borderRadius: 5,
                  background: s.bg,
                  color: s.color,
                  fontSize: 10,
                  fontWeight: 700,
                  textTransform: "uppercase",
                }}
              >
                <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 13 }}>
                  {s.icon}
                </i>
                {s.label}
              </span>
            </div>
            {run.display_title && run.display_title !== run.name && (
              <div className="repository-ellipsis" title={run.display_title} style={{ marginTop: 3, color: "var(--text-tertiary)", fontSize: 11.5 }}>
                {run.display_title}
              </div>
            )}
          </div>
          <button
            type="button"
            aria-label="Close run details"
            onClick={onClose}
            style={{ flexShrink: 0, display: "grid", placeItems: "center", border: "none", borderRadius: 7, padding: 6, background: "transparent", color: "var(--text-secondary)", cursor: "pointer" }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 20 }}>close</i>
          </button>
        </div>

        <div className="repository-run-body">
          <div className="repository-run-grid">
            <Field label="Branch" value={text(run.head_branch)} mono />
            <Field label="Triggering event" value={text(run.event)} />
            <Field label="Commit" value={run.head_sha ? run.head_sha.slice(0, 12) : EMPTY} mono />
            <Field label="Triggered by" value={text(run.actor)} />
            <Field label="Created" value={formatTimestamp(run.created_at)} />
            <Field label="Started" value={formatTimestamp(run.run_started_at)} />
            <Field label="Updated" value={formatTimestamp(run.updated_at)} />
            <Field label="Status" value={run.conclusion ? `${run.status} / ${run.conclusion}` : text(run.status)} />
          </div>

          <a
            href={run.html_url}
            target="_blank"
            rel="noopener noreferrer"
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
              marginTop: 16,
              border: "none",
              borderRadius: 7,
              background: "linear-gradient(135deg,#806cf6,#557ff2)",
              color: "#fff",
              padding: "8px 14px",
              fontSize: 12,
              fontWeight: 700,
              textDecoration: "none",
            }}
          >
            <i className="material-symbols-outlined" aria-hidden="true" style={{ fontSize: 16 }}>open_in_new</i>
            Open on GitHub
          </a>
        </div>
      </div>
    </div>
  );
}
