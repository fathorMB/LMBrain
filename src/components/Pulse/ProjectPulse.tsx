import { useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { buildHandoffPrompt, buildMigrationPrompt } from "../../lib/handoffPrompt";
import { InlineRichText } from "../../lib/inlineRichText";
import { useWikiNavigation } from "../../hooks/useWikiNavigation";
import { InsightReliability } from "../Shared/InsightReliability";
import { PageShell } from "../Shared/PageLayout";
import type { PulseData, Handoff, Adr } from "../../types";

const getMigrationStatusLabelAndColor = (status: string | undefined): { label: string; color: string } => {
  switch (status) {
    case "up-to-date":
      return { label: "Up to date", color: "var(--green)" };
    case "migration-available":
      return { label: "Migration available", color: "var(--yellow)" };
    case "project-newer-than-app":
      return { label: "Project newer than app", color: "var(--red)" };
    case "unknown-project-version":
      return { label: "Unknown project version", color: "var(--text-muted)" };
    case "unknown-bundled-version":
      return { label: "Unknown bundled version", color: "var(--text-muted)" };
    case "migration-guidance-missing":
      return { label: "Guidance missing", color: "var(--yellow)" };
    default:
      return { label: "—", color: "var(--text-muted)" };
  }
};

export function ProjectPulse() {
  const { state, navigateTo } = useWorkspace();
  const [copiedMigration, setCopiedMigration] = useState(false);

  const navigateToWiki = useWikiNavigation();
  const attentionDebts = (state.debts ?? []).filter((debt) =>
    ["open", "planned", "deferred"].includes(debt.status)
    && (["critical", "high"].includes(debt.severity) || !debt.owner)
  );

  const pulse = state.pulseData;
  if (!pulse) {
    return (
      <div
        style={{
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "var(--text-tertiary)",
        }}
      >
        Loading pulse data…
      </div>
    );
  }

  return (
    <PageShell archetype="dense">
      {/* Main column plus a fixed rail: the rail width is content-driven, so
          this page keeps its own grid instead of using CardGrid. */}
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0,1fr) 304px",
          gap: "var(--space-5)",
        }}
      >
        {/* Main column */}
        <div>
          <div
            style={{
              display: "flex",
              alignItems: "flex-end",
              justifyContent: "space-between",
              marginBottom: 6,
            }}
          >
            <div>
              {pulse.milestone && (
                <div
                  style={{
                    fontSize: "var(--text-xs)",
                    letterSpacing: ".1em",
                    textTransform: "uppercase",
                    color: "var(--text-tertiary)",
                    fontWeight: 600,
                    marginBottom: 5,
                  }}
                >
                  <InlineRichText text={pulse.milestone} onWikilinkClick={navigateToWiki} />
                </div>
              )}
              <h1
                style={{
                  fontSize: 30,
                  fontWeight: 800,
                  letterSpacing: "-.03em",
                  margin: 0,
                }}
              >
                Project Pulse
              </h1>
            </div>
          </div>
          {attentionDebts.length > 0 && <button
            type="button"
            onClick={() => navigateTo("debts")}
            style={{
              width: "100%", marginTop: 14, padding: "10px 12px", textAlign: "left",
              border: "1px solid rgba(224,162,58,.35)", borderRadius: 8,
              background: "rgba(224,162,58,.08)", color: "#d9b86d", cursor: "pointer",
            }}
          >
            {attentionDebts.length} active critical/high or untriaged {attentionDebts.length === 1 ? "debt needs" : "debts need"} disposition
          </button>}

          {/* Metrics */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fit, minmax(min(140px, 100%), 1fr))",
              gap: "var(--space-3)",
              marginTop: "var(--space-5)",
              marginBottom: "var(--space-5)",
            }}
          >
            {pulse.metrics.map((m, i) => (
              <MetricCard
                key={i}
                count={m.count}
                label={m.label}
                accent={m.accent}
              />
            ))}
          </div>

          {/* Milestone card */}
          {pulse.milestone && (
            <div
              style={{
                background: "var(--bg-tertiary)",
                border: "1px solid var(--border-secondary)",
                borderRadius: 13,
                padding: "17px 18px",
                marginBottom: 22,
              }}
            >
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  marginBottom: 12,
                }}
              >
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 9,
                  }}
                >
                  <i
                    className="material-symbols-outlined"
                    style={{ fontSize: 18, color: "var(--accent-light)" }}
                  >
                    target
                  </i>
                  <span style={{ fontWeight: 700, fontSize: "var(--text-md)" }}>
                    <InlineRichText text={pulse.milestone} onWikilinkClick={navigateToWiki} />
                  </span>
                </div>
                {pulse.milestone_due && (
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "var(--text-xs)",
                      color: "var(--text-tertiary)",
                    }}
                  >
                    due {pulse.milestone_due}
                  </span>
                )}
              </div>
              {pulse.milestone_progress !== null && (
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 12,
                    marginBottom: 9,
                  }}
                >
                  <div
                    style={{
                      flex: 1,
                      height: 7,
                      background: "#211d27",
                      borderRadius: 5,
                      overflow: "hidden",
                    }}
                  >
                    <div
                      style={{
                        width: `${pulse.milestone_progress}%`,
                        height: "100%",
                        background:
                          "linear-gradient(90deg,#7c6cf6,#9384f8)",
                        borderRadius: 5,
                      }}
                    />
                  </div>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "var(--text-sm)",
                      fontWeight: 600,
                      color: "#cfc9d6",
                    }}
                  >
                    {Math.round(pulse.milestone_progress)}%
                  </span>
                </div>
              )}
            </div>
          )}

          {/* Insight Reliability */}
          <div
            style={{
              background: "var(--bg-tertiary)",
              border: "1px solid var(--border-secondary)",
              borderRadius: 13,
              padding: "17px 18px",
              marginBottom: 22,
            }}
          >
            <InsightReliability />
          </div>

          {/* Actions */}
          {pulse.actions.length > 0 && (
            <>
              <div
                style={{
                  fontSize: "var(--text-xs)",
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "var(--text-tertiary)",
                  fontWeight: 600,
                  marginBottom: 11,
                }}
              >
                Next recommended actions
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 8,
                  marginBottom: 24,
                }}
              >
                {pulse.actions.map((a, i) => (
                  <ActionCard key={i} action={a} />
                ))}
              </div>
            </>
          )}

          {/* Blockers */}
          {pulse.blockers.length > 0 && (
            <>
              <div
                style={{
                  fontSize: "var(--text-xs)",
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "var(--text-tertiary)",
                  fontWeight: 600,
                  marginBottom: 11,
                }}
              >
                Blockers & risks
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 8,
                  marginBottom: 24,
                }}
              >
                {pulse.blockers.map((b, i) => (
                  <div
                    key={i}
                    style={{
                      background: "#16110f",
                      border: "1px solid #3a201c",
                      borderRadius: 12,
                      padding: 14,
                    }}
                  >
                    <div
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 8,
                        marginBottom: 6,
                      }}
                    >
                      <i
                        className="material-symbols-outlined"
                        style={{ fontSize: 16, color: "var(--red)" }}
                      >
                        block
                      </i>
                      <span
                        style={{
                          fontSize: "var(--text-md)",
                          fontWeight: 600,
                          color: "var(--text-primary)",
                        }}
                      >
                        <InlineRichText text={b.title} onWikilinkClick={navigateToWiki} />
                      </span>
                    </div>
                    <div
                      style={{
                        fontSize: "var(--text-sm)",
                        color: "var(--text-secondary)",
                        lineHeight: 1.45,
                      }}
                    >
                      <InlineRichText text={b.description} onWikilinkClick={navigateToWiki} />
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}

          {/* Ready handoffs */}
          {pulse.ready_handoffs.length > 0 && (
            <>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  marginBottom: 11,
                }}
              >
                <div
                  style={{
                    fontSize: "var(--text-xs)",
                    letterSpacing: ".09em",
                    textTransform: "uppercase",
                    color: "var(--text-tertiary)",
                    fontWeight: 600,
                  }}
                >
                  Ready for manual handoff
                </div>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-xs)",
                    color: "var(--text-muted)",
                  }}
                >
                  {pulse.ready_handoffs.length} handoffs
                </span>
              </div>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "1fr 1fr",
                  gap: 11,
                  marginBottom: 24,
                }}
              >
                {pulse.ready_handoffs.map((h, i) => (
                  <HandoffCard key={i} handoff={h} />
                ))}
              </div>
            </>
          )}

          {/* Recent decisions */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: 11,
              marginBottom: 24,
            }}
          >
            <div>
              <div
                style={{
                  fontSize: "var(--text-xs)",
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "var(--text-tertiary)",
                  fontWeight: 600,
                  marginBottom: 11,
                }}
              >
                Recent decisions
              </div>
              <div
                style={{
                  background: "var(--bg-tertiary)",
                  border: "1px solid var(--border-secondary)",
                  borderRadius: 12,
                  overflow: "hidden",
                }}
              >
                {state.adrs.slice(0, 5).map((adr) => (
                  <AdrRow key={adr.id} adr={adr} />
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Right rail */}
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
                      onClick={() => {
                        const prompt = buildMigrationPrompt(
                          state.currentWorkspace!.path,
                          state.currentWorkspace!.project_kit_version || state.currentWorkspace!.kit_version,
                          state.currentWorkspace!.bundled_kit_version,
                          state.currentWorkspace!.kit_migration_status,
                          state.currentWorkspace!.bundled_kit_path
                        );
                        navigator.clipboard.writeText(prompt);
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
      </div>
    </PageShell>
  );
}

// ─── Sub-components ──────────────────────────────────────────────

function MetricCard({
  count,
  label,
  accent,
}: {
  count: number;
  label: string;
  accent: string;
}) {
  return (
    <div
      style={{
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 12,
        padding: 14,
        position: "relative",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: 3,
          height: "100%",
          background: accent,
        }}
      />
      <div
        style={{
          fontSize: 27,
          fontWeight: 800,
          fontFamily: "var(--font-mono)",
          letterSpacing: "-.02em",
        }}
      >
        {count}
      </div>
      <div
        style={{
          fontSize: "var(--text-sm)",
          color: "var(--text-tertiary)",
          marginTop: 2,
        }}
      >
        {label}
      </div>
    </div>
  );
}

function ActionCard({ action }: { action: PulseData["actions"][0] }) {
  const { state } = useWorkspace();
  const [expanded, setExpanded] = useState(false);
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">("idle");
  const navigateToWiki = useWikiNavigation();
  const isHandoff = action.action_type === "handoff" && action.spec_id;
  // Resolve the spec's real (slugged) filename so the handoff path actually exists.
  const specFilename = state.specs
    ?.find((s) => s.id === action.spec_id)
    ?.path.split(/[\\/]/)
    .pop();
  const prompt = isHandoff
    ? buildHandoffPrompt(action.agent, action.spec_id ?? "", "ready", specFilename)
    : null;

  const copyPrompt = async () => {
    if (!prompt) return;
    try {
      await navigator.clipboard.writeText(prompt);
      setCopyState("copied");
    } catch {
      setCopyState("error");
    }
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 13,
        background: "var(--bg-tertiary)",
        border: "1px solid var(--border-secondary)",
        borderRadius: 11,
        padding: "12px 14px",
      }}
    >
      <div
        style={{
          width: 30,
          height: 30,
          borderRadius: 8,
          background: "rgba(124,108,246,.12)",
          border: "1px solid rgba(124,108,246,.24)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flex: "none",
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{ fontSize: 17, color: "var(--accent-light)" }}
        >
          rocket_launch
        </i>
      </div>
      <div style={{ flex: 1 }}>
        <div
          style={{
            fontSize: "var(--text-md)",
            fontWeight: 600,
            color: "var(--text-primary)",
          }}
        >
          <InlineRichText text={action.title} onWikilinkClick={navigateToWiki} />
        </div>
        <div
          style={{
            fontSize: "var(--text-sm)",
            color: "var(--text-tertiary)",
          }}
        >
          <InlineRichText text={action.description} onWikilinkClick={navigateToWiki} />
        </div>
        {expanded && prompt && (
          <div style={{ marginTop: 10 }}>
            <div
              style={{
                fontSize: "var(--text-xs)",
                color: "#7fa8f5",
                marginBottom: 6,
                lineHeight: 1.4,
              }}
            >
              <i
                className="material-symbols-outlined"
                style={{ fontSize: 13, verticalAlign: "middle", marginRight: 4 }}
              >
                lightbulb
              </i>
              The prompt includes v3 context-economy guidance. The agent will use{" "}
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)" }}>
                lmbrain_spec_context
              </span>{" "}
              for a compact handoff context before expanding to full artifacts.
            </div>
            <textarea
              aria-label={`Handoff prompt for ${action.spec_id}`}
              readOnly
              value={prompt}
              onClick={(event) => event.currentTarget.select()}
              style={{
                width: "100%",
                minHeight: 76,
                resize: "vertical",
                background: "var(--bg-primary)",
                border: "1px solid var(--border-primary)",
                borderRadius: 6,
                padding: 8,
                color: "var(--text-secondary)",
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-xs)",
              }}
            />
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 8 }}>
              <button
                type="button"
                onClick={copyPrompt}
                style={{
                  background: "var(--accent-light)",
                  color: "#fff",
                  border: "none",
                  borderRadius: 6,
                  padding: "6px 12px",
                  fontSize: "var(--text-xs)",
                  fontWeight: 600,
                  cursor: "pointer",
                  display: "inline-flex",
                  alignItems: "center",
                  gap: 5,
                }}
              >
                <i className="material-symbols-outlined" style={{ fontSize: 14 }}>
                  content_copy
                </i>
                Copy prompt
              </button>
              {copyState === "copied" && (
                <span role="status" style={{ fontSize: "var(--text-xs)", color: "var(--green)" }}>
                  Copied to clipboard.
                </span>
              )}
              {copyState === "error" && (
                <span role="alert" style={{ fontSize: "var(--text-xs)", color: "#e0584a" }}>
                  Could not copy the prompt. Select and copy it manually.
                </span>
              )}
            </div>
          </div>
        )}
      </div>
      {isHandoff && (
        <button
          type="button"
          onClick={() => setExpanded((value) => !value)}
          aria-expanded={expanded}
          style={{
            background: "rgba(255,255,255,0.06)",
            border: "1px solid rgba(255,255,255,0.1)",
            borderRadius: 6,
            padding: "5px 10px",
            fontSize: "var(--text-xs)",
            color: "#fff",
            cursor: "pointer",
            display: "inline-flex",
            alignItems: "center",
            gap: 4,
            flex: "none",
            alignSelf: "flex-start",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
            {expanded ? "visibility_off" : "visibility"}
          </i>
          {expanded ? "Hide prompt" : "View prompt"}
        </button>
      )}
    </div>
  );
}

function HandoffCard({ handoff }: { handoff: Handoff }) {
  const { openDetailArtifact } = useWorkspace();
  return (
    <div
      style={{
        background: "var(--bg-tertiary)",
        border: "1px solid #2a2731",
        borderRadius: 12,
        padding: 15,
        borderTop: "2px solid var(--accent)",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          marginBottom: 8,
        }}
      >
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "var(--text-sm)",
            color: "#bcaef6",
            fontWeight: 500,
          }}
        >
          {handoff.id}
        </span>
      </div>
      <div
        style={{
          fontSize: "var(--text-md)",
          fontWeight: 700,
          marginBottom: 10,
          color: "var(--text-primary)",
        }}
      >
        {handoff.title}
      </div>
      <button
        onClick={() => openDetailArtifact({ title: handoff.title, path: handoff.path })}
        style={{
          width: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          gap: 7,
          background: "linear-gradient(180deg,#8676f7,#6e5bf2)",
          border: "none",
          color: "#fff",
          borderRadius: 8,
          padding: 8,
          fontSize: "var(--text-sm)",
          fontWeight: 600,
          cursor: "pointer",
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 16 }}>
          open_in_full
        </i>
        Open handoff
      </button>
    </div>
  );
}

function AdrRow({ adr }: { adr: Adr }) {
  const { openDetailArtifact } = useWorkspace();
  const statusColors: Record<string, { color: string; bg: string }> = {
    accepted: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    proposed: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
    superseded: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
    deprecated: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
  };
  const sc = statusColors[adr.status] || statusColors.proposed;

  return (
    <div
      onClick={() => openDetailArtifact({ title: adr.title, path: adr.path })}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "11px 13px",
        borderBottom: "1px solid #201d26",
        cursor: "pointer",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.background = "#181520";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.background = "transparent";
      }}
    >
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-xs)",
          color: "#bcaef6",
        }}
      >
        {adr.id}
      </span>
      <span
        style={{
          flex: 1,
          fontSize: "var(--text-sm)",
          color: "var(--text-primary)",
        }}
      >
        {adr.title}
      </span>
      <span
        style={{
          fontSize: "var(--text-2xs)",
          fontWeight: 600,
          color: sc.color,
          background: sc.bg,
          borderRadius: 4,
          padding: "1px 6px",
        }}
      >
        {adr.status.toUpperCase()}
      </span>
    </div>
  );
}

function MetaRow({
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

function QuickLink({
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
