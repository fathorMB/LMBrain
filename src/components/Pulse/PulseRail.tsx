import { useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { buildMigrationPrompt } from "../../lib/handoffPrompt";
import { getMigrationStatusLabelAndColor } from "../../lib/migrationStatus";

export function MetaRow({
  label,
  value,
  mono,
  accent,
}: {
  label: string;
  value: string;
  mono?: boolean;
  accent?: string;
}) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
      }}
    >
      <span style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>
        {label}
      </span>
      <span
        style={{
          fontSize: mono ? 12 : 12.5,
          fontWeight: mono ? 400 : 600,
          fontFamily: mono ? "var(--font-mono)" : "inherit",
          color: accent || "#cfc9d6",
          display: "flex",
          alignItems: "center",
          gap: 5,
        }}
      >
        {accent && (
          <span
            style={{
              width: 6,
              height: 6,
              borderRadius: "50%",
              background: accent,
            }}
          />
        )}
        {value}
      </span>
    </div>
  );
}

export function QuickLink({
  icon,
  label,
  documentPath,
}: {
  icon: string;
  label: string;
  documentPath?: string;
}) {
  const { state, openDetailArtifact } = useWorkspace();
  const openDocument = () => {
    if (!state.currentWorkspace || !documentPath) return;
    openDetailArtifact({
      title: label,
      path: `${state.currentWorkspace.path}/${documentPath}`,
    });
  };

  return (
    <button
      type="button"
      onClick={openDocument}
      disabled={!documentPath}
      aria-label={`Open ${label}`}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "9px 11px",
        background: "#100e14",
        border: "1px solid #221f29",
        borderRadius: 9,
        cursor: "pointer",
        textAlign: "left",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor = "#36303f";
        e.currentTarget.style.background = "#161320";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor = "#221f29";
        e.currentTarget.style.background = "#100e14";
      }}
    >
      <i
        className="material-symbols-outlined"
        style={{ fontSize: 17, color: "var(--accent-light)" }}
      >
        {icon}
      </i>
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-sm)",
          flex: 1,
          color: "var(--text-primary)",
        }}
      >
        {label}
      </span>
      <i
        className="material-symbols-outlined"
        style={{ fontSize: 15, color: "var(--text-tertiary)" }}
      >
        north_east
      </i>
    </button>
  );
}

export function PulseRail() {
  const { state } = useWorkspace();
  const [copiedMigration, setCopiedMigration] = useState(false);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
      {/* Project metadata */}
      <div
        style={{
          background: "var(--bg-tertiary)",
          border: "1px solid var(--border-secondary)",
          borderRadius: 13,
          padding: 15,
        }}
      >
        <div
          style={{
            fontSize: "var(--text-xs)",
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
            fontWeight: 600,
            marginBottom: 13,
          }}
        >
          Project metadata
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          <MetaRow
            label="Repository"
            value={state.currentWorkspace?.name || "—"}
          />
          <MetaRow
            label="Branch"
            value={state.gitInfo?.branch || "—"}
            mono
          />
          <MetaRow
            label="Path"
            value={state.currentWorkspace?.path || "—"}
            mono
          />
          <MetaRow
            label=".lmbrain version"
            value={state.currentWorkspace?.project_kit_version || state.currentWorkspace?.kit_version || "—"}
            mono
          />
          {state.currentWorkspace && state.currentWorkspace.bundled_kit_version && (
            <>
              <MetaRow
                label="Bundled kit"
                value={state.currentWorkspace.bundled_kit_version}
                mono
              />
              <MetaRow
                label="Kit status"
                value={getMigrationStatusLabelAndColor(state.currentWorkspace.kit_migration_status).label}
                accent={getMigrationStatusLabelAndColor(state.currentWorkspace.kit_migration_status).color}
              />
            </>
          )}
          <div
            style={{
              height: 1,
              background: "#201d26",
              margin: "2px 0",
            }}
          />
          <MetaRow
            label="Specs"
            value={`${state.currentWorkspace?.spec_count || 0}`}
          />
          <MetaRow
            label="Decisions"
            value={String(state.currentWorkspace?.decision_count || 0)}
          />
          <MetaRow
            label="Watcher"
            value={state.watcherActive ? "active" : "inactive"}
            accent={state.watcherActive ? "var(--green)" : "var(--text-muted)"}
          />
          {state.currentWorkspace &&
            state.currentWorkspace.kit_migration_status &&
            state.currentWorkspace.kit_migration_status !== "up-to-date" &&
            state.currentWorkspace.kit_migration_status !== "project-newer-than-app" && (
              <div style={{ marginTop: 10 }}>
                <button
                  type="button"
                  onClick={() => {
                    const prompt = buildMigrationPrompt(
                      state.currentWorkspace!.path,
                      state.currentWorkspace!.project_kit_version || state.currentWorkspace!.kit_version,
                      state.currentWorkspace!.bundled_kit_version,
                      state.currentWorkspace!.kit_migration_status,
                      state.currentWorkspace!.bundled_kit_path
                    );
                    navigator.clipboard?.writeText(prompt);
                    setCopiedMigration(true);
                    setTimeout(() => setCopiedMigration(false), 2000);
                  }}
                  style={{
                    width: "100%",
                    background: "rgba(124, 108, 246, 0.1)",
                    border: "1px solid var(--accent)",
                    borderRadius: 8,
                    padding: "8px 12px",
                    fontSize: "var(--text-xs)",
                    color: "var(--accent-light)",
                    fontWeight: 600,
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    gap: 6,
                  }}
                >
                  <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
                    {copiedMigration ? "check" : "content_copy"}
                  </i>
                  {copiedMigration ? "Copied!" : "Copy migration prompt"}
                </button>
              </div>
            )}
        </div>
      </div>

      {/* Quick links */}
      <div
        style={{
          background: "var(--bg-tertiary)",
          border: "1px solid var(--border-secondary)",
          borderRadius: 13,
          padding: 15,
        }}
      >
        <div
          style={{
            fontSize: "var(--text-xs)",
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
            fontWeight: 600,
            marginBottom: 12,
          }}
        >
          Quick links
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
          <QuickLink icon="description" label="STATUS.md" documentPath=".lmbrain/STATUS.md" />
          <QuickLink icon="description" label="ROADMAP.md" documentPath=".lmbrain/ROADMAP.md" />
          {state.handoffs.filter((h) => h.status === "ready").length >
            0 && (
            <QuickLink
              icon="swap_horiz"
              label={
                state.handoffs.find((h) => h.status === "ready")?.id ||
                "HANDOFF"
              }
            />
          )}
        </div>
      </div>

      {/* Agents */}
      <div
        style={{
          background: "var(--bg-tertiary)",
          border: "1px solid var(--border-secondary)",
          borderRadius: 13,
          padding: 15,
        }}
      >
        <div
          style={{
            fontSize: "var(--text-xs)",
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
            fontWeight: 600,
            marginBottom: 12,
          }}
        >
          Agents (manual start)
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          {state.agents.map((agent) => (
            <div
              key={agent.id}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 9,
              }}
            >
              <div
                style={{
                  width: 26,
                  height: 26,
                  borderRadius: 7,
                  background: "rgba(124,108,246,.12)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <i
                  className="material-symbols-outlined"
                  style={{ fontSize: 15, color: "var(--accent-light)" }}
                >
                  strategy
                </i>
              </div>
              <div style={{ flex: 1 }}>
                <div
                  style={{
                    fontSize: "var(--text-sm)",
                    fontWeight: 600,
                    color: "var(--text-primary)",
                  }}
                >
                  {agent.title}
                </div>
                <div
                  style={{
                    fontSize: "var(--text-xs)",
                    color: "var(--text-tertiary)",
                  }}
                >
                  {agent.role || agent.status}
                </div>
              </div>
            </div>
          ))}
        </div>
        <div
          style={{
            marginTop: 12,
            fontSize: "var(--text-xs)",
            color: "var(--text-tertiary)",
            display: "flex",
            alignItems: "center",
            gap: 6,
            lineHeight: 1.4,
          }}
        >
          <i
            className="material-symbols-outlined"
            style={{ fontSize: 14, color: "var(--text-tertiary)" }}
          >
            info
          </i>
          LMBrain never auto-starts agents.
        </div>
      </div>
    </div>
  );
}
