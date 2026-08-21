import { useWorkspace } from "../../hooks/useWorkspace";
import { InlineRichText } from "../../lib/inlineRichText";
import { useWikiNavigation } from "../../hooks/useWikiNavigation";
import { InsightReliability } from "../Shared/InsightReliability";
import { PageShell } from "../Shared/PageLayout";
import { PulseMetricCard } from "./PulseMetricCard";
import { PulseActionCard } from "./PulseActionCard";
import { PulseHandoffCard } from "./PulseHandoffCard";
import { PulseAdrRow } from "./PulseAdrRow";
import { PulseRail } from "./PulseRail";

export function ProjectPulse() {
  const { state, navigateTo } = useWorkspace();
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
      {/* Main column plus a fixed rail */}
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

          {attentionDebts.length > 0 && (
            <button
              type="button"
              onClick={() => navigateTo("debts")}
              style={{
                width: "100%",
                marginTop: 14,
                padding: "10px 12px",
                textAlign: "left",
                border: "1px solid rgba(224,162,58,.35)",
                borderRadius: 8,
                background: "rgba(224,162,58,.08)",
                color: "#d9b86d",
                cursor: "pointer",
              }}
            >
              {attentionDebts.length} active critical/high or untriaged {attentionDebts.length === 1 ? "debt needs" : "debts need"} disposition
            </button>
          )}

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
              <PulseMetricCard
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
                        background: "linear-gradient(90deg,#7c6cf6,#9384f8)",
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
                  <PulseActionCard key={i} action={a} />
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
                  <PulseHandoffCard key={i} handoff={h} />
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
                  <PulseAdrRow key={adr.id} adr={adr} />
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Right rail */}
        <PulseRail />
      </div>
    </PageShell>
  );
}
