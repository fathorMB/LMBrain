import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getPulseData, getAdrs, getHandoffs, getAgents, getDiagnostics } from "../../lib/commands";
import { buildHandoffPrompt } from "../../lib/handoffPrompt";
import { InlineRichText } from "../../lib/inlineRichText";
import { useWikiNavigation } from "../../hooks/useWikiNavigation";
import type { PulseData, Handoff, Adr, KitDiagnostic } from "../../types";

function generateFixPrompt(d: KitDiagnostic): string {
  const path = d.path || "unknown file";
  const msg = d.message;
  
  if (msg.toLowerCase().includes("malformed") || msg.toLowerCase().includes("yaml") || msg.toLowerCase().includes("frontmatter")) {
    return `Please fix the malformed frontmatter in the file: ${path}
The parser error message is: ${msg}

Instructions:
1. Fix the frontmatter block at the top of the file so that it is valid YAML.
2. Make sure any values containing a colon are enclosed in quotes (e.g. title: "React UI: list").
3. Ensure that references use bare IDs rather than [[wikilinks]].
4. Do not modify the body content of the file or any other files.
5. Preserve all intended field values.`;
  }
  
  if (msg.toLowerCase().includes("mismatch") || msg.toLowerCase().includes("status")) {
    return `There is a status mismatch in the file: ${path}
The status in the frontmatter does not match its directory location.
Conflict details: ${msg}

Instructions:
Please align the status:
Either:
- Update the status field in the frontmatter to match the folder the file resides in.
Or:
- Move the file to the folder corresponding to its frontmatter status.
Do not make any other changes to the file or its body.`;
  }

  return `Please fix the issue in the file: ${path}
Error details: ${msg}

Instructions:
Resolve the reported error while preserving the rest of the file content and structure.`;
}

export function ProjectPulse() {
  const { state, dispatch } = useWorkspace();
  const [diagnostics, setDiagnostics] = useState<KitDiagnostic[]>([]);
  const [expandedDiagnostic, setExpandedDiagnostic] = useState<number | null>(null);
  const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const [pulse, adrs, handoffs, agents, diags] = await Promise.all([
          getPulseData(),
          getAdrs(),
          getHandoffs(),
          getAgents(),
          getDiagnostics(),
        ]);
        dispatch({ type: "SET_PULSE", data: pulse });
        dispatch({ type: "SET_ADRS", adrs });
        dispatch({ type: "SET_HANDOFFS", handoffs });
        dispatch({ type: "SET_AGENTS", agents });
        setDiagnostics(diags);
      } catch (err) {
        console.error("Failed to load pulse data:", err);
      }
    };
    load();
  }, [dispatch]);

  const navigateToWiki = useWikiNavigation();

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
    <div style={{ padding: "26px 30px 60px" }}>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "minmax(0,1fr) 304px",
          gap: 24,
          maxWidth: 1320,
          margin: "0 auto",
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
                    fontSize: 11,
                    letterSpacing: ".1em",
                    textTransform: "uppercase",
                    color: "#6c6671",
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

          {pulse.focus && (
            <p
              style={{
                fontSize: 14.5,
                lineHeight: 1.5,
                color: "#b6b1bb",
                margin: "8px 0 22px",
                maxWidth: 680,
              }}
            >
              Current focus:{" "}
              <span style={{ color: "var(--text-primary)", fontWeight: 600 }}>
                <InlineRichText text={pulse.focus} onWikilinkClick={navigateToWiki} />
              </span>
            </p>
          )}

          {/* Metrics */}
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(4,1fr)",
              gap: 11,
              marginBottom: 22,
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
                  <span style={{ fontWeight: 700, fontSize: 14.5 }}>
                    <InlineRichText text={pulse.milestone} onWikilinkClick={navigateToWiki} />
                  </span>
                </div>
                {pulse.milestone_due && (
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: 11.5,
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
                      fontSize: 12,
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

          {/* Diagnostics */}
          {diagnostics.length > 0 && (
            <>
              <div
                style={{
                  fontSize: 11,
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "#e0a23a",
                  fontWeight: 600,
                  marginBottom: 11,
                  display: "flex",
                  alignItems: "center",
                  gap: 6,
                }}
              >
                <i className="material-symbols-outlined" style={{ fontSize: 14 }}>
                  warning
                </i>
                Diagnostics ({diagnostics.length})
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 6,
                  marginBottom: 24,
                }}
              >
                {diagnostics.map((d, i) => (
                  <div
                    key={i}
                    style={{
                      display: "flex",
                      alignItems: "flex-start",
                      gap: 10,
                      background:
                        d.severity === "error"
                          ? "rgba(224,88,74,.08)"
                          : "rgba(224,162,58,.08)",
                      border: `1px solid ${
                        d.severity === "error"
                          ? "rgba(224,88,74,.2)"
                          : "rgba(224,162,58,.2)"
                      }`,
                      borderRadius: 10,
                      padding: "10px 13px",
                      fontSize: 12.5,
                      color: "#c2bdc8",
                      lineHeight: 1.5,
                    }}
                  >
                    <i
                      className="material-symbols-outlined"
                      style={{
                        fontSize: 16,
                        flex: "none",
                        marginTop: 1,
                        color:
                          d.severity === "error"
                            ? "var(--red)"
                            : "var(--yellow)",
                      }}
                    >
                      {d.severity === "error" ? "error" : "warning"}
                    </i>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div>{d.message}</div>
                      {d.path && (
                        <div
                          style={{
                            fontFamily: "var(--font-mono)",
                            fontSize: 11,
                            color: "#6c6671",
                            marginTop: 3,
                          }}
                        >
                          {d.path}
                        </div>
                      )}
                      {expandedDiagnostic === i && (
                        <div
                          style={{
                            marginTop: 10,
                            display: "flex",
                            flexDirection: "column",
                            gap: 8,
                            width: "100%",
                          }}
                        >
                          <textarea
                            readOnly
                            value={generateFixPrompt(d)}
                            style={{
                              width: "100%",
                              height: 120,
                              background: "#16131c",
                              border: "1px solid #2e2838",
                              borderRadius: 6,
                              padding: 8,
                              color: "#c2bdc8",
                              fontFamily: "var(--font-mono)",
                              fontSize: 11,
                              resize: "none",
                            }}
                            onClick={(e) => (e.target as HTMLTextAreaElement).select()}
                          />
                          <button
                            onClick={() => {
                              navigator.clipboard.writeText(generateFixPrompt(d));
                              setCopiedIndex(i);
                              setTimeout(() => setCopiedIndex(null), 2000);
                            }}
                            style={{
                              background: "var(--accent-light)",
                              color: "#fff",
                              border: "none",
                              borderRadius: 6,
                              padding: "6px 12px",
                              fontSize: 11.5,
                              fontWeight: 600,
                              cursor: "pointer",
                              display: "inline-flex",
                              alignItems: "center",
                              gap: 5,
                              alignSelf: "flex-start",
                            }}
                          >
                            <i className="material-symbols-outlined" style={{ fontSize: 14 }}>
                              {copiedIndex === i ? "check" : "content_copy"}
                            </i>
                            {copiedIndex === i ? "Copied!" : "Copy fix prompt"}
                          </button>
                        </div>
                      )}
                    </div>
                    <button
                      onClick={() => setExpandedDiagnostic(expandedDiagnostic === i ? null : i)}
                      style={{
                        background: "rgba(255,255,255,0.06)",
                        border: "1px solid rgba(255,255,255,0.1)",
                        borderRadius: 6,
                        padding: "3px 8px",
                        fontSize: 11,
                        color: "#fff",
                        cursor: "pointer",
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 4,
                        marginLeft: "auto",
                        flexShrink: 0,
                        marginTop: 2,
                      }}
                    >
                      <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
                        build
                      </i>
                      {expandedDiagnostic === i ? "Hide" : "Fix"}
                    </button>
                  </div>
                ))}
              </div>
            </>
          )}

          {/* Actions */}
          {pulse.actions.length > 0 && (
            <>
              <div
                style={{
                  fontSize: 11,
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "#6c6671",
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
                  fontSize: 11,
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "#6c6671",
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
                          fontSize: 13,
                          fontWeight: 600,
                          color: "var(--text-primary)",
                        }}
                      >
                        <InlineRichText text={b.title} onWikilinkClick={navigateToWiki} />
                      </span>
                    </div>
                    <div
                      style={{
                        fontSize: 12,
                        color: "#9a949f",
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
                    fontSize: 11,
                    letterSpacing: ".09em",
                    textTransform: "uppercase",
                    color: "#6c6671",
                    fontWeight: 600,
                  }}
                >
                  Ready for manual handoff
                </div>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: 11,
                    color: "#56525b",
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
                  fontSize: 11,
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "#6c6671",
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
                fontSize: 11,
                letterSpacing: ".09em",
                textTransform: "uppercase",
                color: "#6c6671",
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
                value={state.currentWorkspace?.kit_version || "—"}
                mono
              />
              <div
                style={{
                  height: 1,
                  background: "#201d26",
                  margin: "2px 0",
                }}
              />
              <MetaRow
                label="Specs / Tasks"
                value={`${state.currentWorkspace?.spec_count || 0} / ${state.currentWorkspace?.task_count || 0}`}
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
                fontSize: 11,
                letterSpacing: ".09em",
                textTransform: "uppercase",
                color: "#6c6671",
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
                fontSize: 11,
                letterSpacing: ".09em",
                textTransform: "uppercase",
                color: "#6c6671",
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
                        fontSize: 12.5,
                        fontWeight: 600,
                        color: "var(--text-primary)",
                      }}
                    >
                      {agent.title}
                    </div>
                    <div
                      style={{
                        fontSize: 11,
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
                fontSize: 11,
                color: "#6c6671",
                display: "flex",
                alignItems: "center",
                gap: 6,
                lineHeight: 1.4,
              }}
            >
              <i
                className="material-symbols-outlined"
                style={{ fontSize: 14, color: "#6c6671" }}
              >
                info
              </i>
              LMBrain never auto-starts agents.
            </div>
          </div>
        </div>
      </div>
    </div>
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
          fontSize: 12,
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
            fontSize: 13.5,
            fontWeight: 600,
            color: "var(--text-primary)",
          }}
        >
          <InlineRichText text={action.title} onWikilinkClick={navigateToWiki} />
        </div>
        <div
          style={{
            fontSize: 12,
            color: "var(--text-tertiary)",
          }}
        >
          <InlineRichText text={action.description} onWikilinkClick={navigateToWiki} />
        </div>
        {expanded && prompt && (
          <div style={{ marginTop: 10 }}>
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
                fontSize: 11.5,
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
                  fontSize: 11.5,
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
                <span role="status" style={{ fontSize: 11, color: "var(--green)" }}>
                  Copied to clipboard.
                </span>
              )}
              {copyState === "error" && (
                <span role="alert" style={{ fontSize: 11, color: "#e0584a" }}>
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
            fontSize: 11,
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
  const { dispatch } = useWorkspace();
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
            fontSize: 12,
            color: "#bcaef6",
            fontWeight: 500,
          }}
        >
          {handoff.id}
        </span>
      </div>
      <div
        style={{
          fontSize: 14,
          fontWeight: 700,
          marginBottom: 10,
          color: "var(--text-primary)",
        }}
      >
        {handoff.title}
      </div>
      <button
        onClick={() => dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: { title: handoff.title, path: handoff.path } })}
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
          fontSize: 12.5,
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
  const { dispatch } = useWorkspace();
  const statusColors: Record<string, { color: string; bg: string }> = {
    accepted: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
    proposed: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
    superseded: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
    deprecated: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
  };
  const sc = statusColors[adr.status] || statusColors.proposed;

  return (
    <div
      onClick={() => dispatch({ type: "SET_DETAIL_ARTIFACT", artifact: { title: adr.title, path: adr.path } })}
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
          fontSize: 11.5,
          color: "#bcaef6",
        }}
      >
        {adr.id}
      </span>
      <span
        style={{
          flex: 1,
          fontSize: 12.5,
          color: "var(--text-primary)",
        }}
      >
        {adr.title}
      </span>
      <span
        style={{
          fontSize: 10,
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
      <span style={{ fontSize: 12, color: "var(--text-tertiary)" }}>
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
  const { state, dispatch } = useWorkspace();
  const openDocument = () => {
    if (!state.currentWorkspace || !documentPath) return;
    dispatch({
      type: "SET_DETAIL_ARTIFACT",
      artifact: {
        title: label,
        path: `${state.currentWorkspace.path}/${documentPath}`,
      },
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
          fontSize: 12,
          flex: 1,
          color: "var(--text-primary)",
        }}
      >
        {label}
      </span>
      <i
        className="material-symbols-outlined"
        style={{ fontSize: 15, color: "#6c6671" }}
      >
        north_east
      </i>
    </button>
  );
}
