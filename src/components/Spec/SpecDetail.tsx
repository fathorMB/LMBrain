import { useWorkspace } from "../../hooks/useWorkspace";
import { MarkdownRenderer } from "../../lib/markdown";
import type { Spec } from "../../types";
import { buildHandoffPrompt } from "../../lib/handoffPrompt";
import { OperatorVerificationPanel } from "./OperatorVerificationPanel";

export function SpecDetail() {
  const { state, closeSpecDetail, loadAllData, navigateTo } = useWorkspace();
  const specs = state.specs;
  const readySpecs = specs.filter((s) => s.status === "ready");

  // Show first ready spec, or first spec
  const spec = state.selectedSpec || readySpecs[0] || specs[0];
  const relatedFindings = spec ? (state.findings ?? []).filter((finding) =>
    ["open", "planned", "deferred"].includes(finding.status)
    && (finding.origin_artifact === spec.id
      || finding.related_specs.includes(spec.id)
      || finding.target_specs.includes(spec.id))
  ) : [];
  const directDependencies = spec
    ? (spec.depends_on ?? []).map((id) => ({
        id,
        spec: specs.find((candidate) => candidate.id === id),
      }))
    : [];
  const dependencyBlockers = directDependencies.filter(
    (dependency) => dependency.spec?.status !== "done",
  );
  const latestParking = spec?.parking_events?.at(-1);

  if (!spec) {
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
        No specifications found.
      </div>
    );
  }

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 880, margin: "0 auto", padding: "22px 36px 70px" }}>
        {/* Breadcrumb */}
        <button
          type="button"
          aria-label="Back to specification board"
          onClick={closeSpecDetail}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontFamily: "var(--font-mono)",
            fontSize: 11.5,
            color: "#6c6671",
            marginBottom: 18,
            cursor: "pointer",
            width: "max-content",
            padding: 0,
            border: 0,
            background: "transparent",
            textAlign: "left",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
            arrow_back
          </i>
          specs / {spec.status} / {spec.id}.md
        </button>

        {/* Header */}
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            gap: 20,
            marginBottom: 18,
          }}
        >
          <div>
            <div
              style={{
                display: "flex",
                alignItems: "center",
                gap: 10,
                marginBottom: 8,
              }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: 13,
                  color: "#bcaef6",
                  fontWeight: 500,
                }}
              >
                {spec.id}
              </span>
              {spec.status === "ready" && (
                <span
                  style={{
                    display: "inline-flex",
                    alignItems: "center",
                    gap: 5,
                    fontSize: 11,
                    fontWeight: 700,
                    color: "var(--accent)",
                    background: "rgba(124,108,246,.13)",
                    border: "1px solid rgba(124,108,246,.3)",
                    borderRadius: 6,
                    padding: "3px 9px",
                  }}
                >
                  <span
                    style={{
                      width: 6,
                      height: 6,
                      borderRadius: "50%",
                      background: "var(--accent)",
                    }}
                  />
                  READY FOR HANDOFF
                </span>
              )}
            </div>
            <h1
              style={{
                fontSize: 27,
                fontWeight: 800,
                letterSpacing: "-.028em",
                margin: 0,
              }}
            >
              {spec.title}
            </h1>
          </div>
        </div>

        {/* Meta row */}
        <div
          style={{
            display: "flex",
            gap: 9,
            marginBottom: 22,
            flexWrap: "wrap",
          }}
        >
          {spec.recommended_agent && (
            <MetaPill
              icon="smart_toy"
              label="Recommended agent"
              value={spec.recommended_agent}
            />
          )}
          {spec.priority && (
            <MetaPill
              icon="priority_high"
              label="Priority"
              value={spec.priority}
            />
          )}
          {spec.milestone && (
            <MetaPill
              icon="target"
              label="Milestone"
              value={spec.milestone}
            />
          )}
        </div>
        {relatedFindings.length > 0 && <button
          type="button"
          onClick={() => navigateTo("findings")}
          style={{
            width: "100%", marginBottom: 18, padding: "10px 12px", textAlign: "left",
            border: "1px solid rgba(224,162,58,.35)", borderRadius: 8,
            background: "rgba(224,162,58,.08)", color: "#d9b86d", cursor: "pointer",
          }}
        >
          {relatedFindings.length} active {relatedFindings.length === 1 ? "finding" : "findings"} related to this spec · Open Findings
        </button>}
        {directDependencies.length > 0 && (
          <section
            aria-label="Hard spec dependencies"
            style={{
              marginBottom: 18,
              padding: "12px 14px",
              border: `1px solid ${dependencyBlockers.length ? "rgba(224,162,58,.4)" : "var(--border-primary)"}`,
              borderRadius: 8,
              background: dependencyBlockers.length ? "rgba(224,162,58,.06)" : "var(--bg-tertiary)",
            }}
          >
            <div style={{ fontSize: 12, fontWeight: 700, marginBottom: 8 }}>
              Hard prerequisites
              {dependencyBlockers.length > 0 && (
                <span style={{ marginLeft: 8, color: "#d9b86d" }}>
                  {dependencyBlockers.length} incomplete
                </span>
              )}
            </div>
            {directDependencies.map((dependency) => (
              <div
                key={dependency.id}
                style={{ fontFamily: "var(--font-mono)", fontSize: 11, marginTop: 5 }}
              >
                {dependency.id} · {dependency.spec?.status ?? "missing"}
              </div>
            ))}
            <div style={{ marginTop: 9, fontSize: 11, color: "var(--text-tertiary)" }}>
              Dependency lifecycle changes are intentionally unavailable in the app.
            </div>
          </section>
        )}
        {spec.status === "backlog" && latestParking && (
          <section
            aria-label="Parking history"
            style={{
              marginBottom: 18,
              padding: "12px 14px",
              border: "1px solid var(--border-primary)",
              borderRadius: 8,
              background: "var(--bg-tertiary)",
            }}
          >
            <div style={{ fontSize: 12, fontWeight: 700 }}>Previously parked in backlog</div>
            <div style={{ marginTop: 7, fontSize: 12 }}>{latestParking.reason}</div>
            <div style={{ marginTop: 5, fontSize: 11, color: "var(--text-tertiary)" }}>
              {latestParking.actor} · {latestParking.timestamp}
              {latestParking.revisit_condition
                ? ` · Revisit: ${latestParking.revisit_condition}`
                : ""}
            </div>
            <div style={{ marginTop: 8, fontSize: 11, color: "var(--text-tertiary)" }}>
              Re-approval and lifecycle actions are intentionally unavailable in the app.
            </div>
          </section>
        )}

        {/* Lifecycle rail */}
        <LifecycleRail status={spec.status} />

        {/* Handoff CTA for ready specs */}
        {spec.status === "ready" && (
          <HandoffCTA spec={spec} />
        )}

        {(spec.status === "review" || spec.status === "done") && (
          <OperatorVerificationPanel
            spec={spec}
            onAttested={loadAllData}
          />
        )}

        {/* Body */}
        <MarkdownRenderer content={spec.body} />
      </div>
    </div>
  );
}

function MetaPill({
  icon,
  label,
  value,
}: {
  icon: string;
  label: string;
  value: string;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        background: "var(--bg-tertiary)",
        border: "1px solid #262330",
        borderRadius: 9,
        padding: "8px 12px",
      }}
    >
      <i
        className="material-symbols-outlined"
        style={{ fontSize: 16, color: "var(--accent-light)" }}
      >
        {icon}
      </i>
      <div>
        <div
          style={{
            fontSize: 10,
            color: "#6c6671",
            textTransform: "uppercase",
            letterSpacing: ".06em",
          }}
        >
          {label}
        </div>
        <div style={{ fontSize: 12.5, fontWeight: 600 }}>{value}</div>
      </div>
    </div>
  );
}

function LifecycleRail({ status }: { status: string }) {
  const stages = [
    "proposed",
    "ready",
    "in-progress",
    "review",
    "accepted",
  ];
  const icons: Record<string, string> = {
    proposed: "check",
    ready: "flag",
    "in-progress": "bolt",
    review: "rate_review",
    accepted: "verified",
  };
  const currentIdx = stages.indexOf(status);

  return (
    <div
      style={{
        background: "#100e14",
        border: "1px solid #201d26",
        borderRadius: 12,
        padding: "16px 18px",
        marginBottom: 18,
      }}
    >
      <div
        style={{
          fontSize: 10.5,
          letterSpacing: ".09em",
          textTransform: "uppercase",
          color: "#6c6671",
          fontWeight: 600,
          marginBottom: 14,
        }}
      >
        Lifecycle
      </div>
      <div style={{ display: "flex", alignItems: "center" }}>
        {stages.map((stage, i) => {
          const isActive = i <= currentIdx;
          const isCurrent = i === currentIdx;
          return (
            <div key={stage} style={{ display: "flex", alignItems: "center", flex: 1 }}>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  alignItems: "center",
                  gap: 7,
                  flex: "none",
                }}
              >
                <div
                  style={{
                    width: isCurrent ? 30 : 26,
                    height: isCurrent ? 30 : 26,
                    borderRadius: "50%",
                    background: isActive
                      ? isCurrent
                        ? "var(--accent)"
                        : "rgba(124,108,246,.15)"
                      : "#15131a",
                    border: isActive
                      ? isCurrent
                        ? "none"
                        : "1.5px solid var(--accent)"
                      : "1.5px solid #2e2a36",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    boxShadow: isCurrent
                      ? "0 0 0 4px rgba(124,108,246,.18)"
                      : "none",
                  }}
                >
                  <i
                    className="material-symbols-outlined"
                    style={{
                      fontSize: isCurrent ? 17 : 15,
                      color: isActive
                        ? isCurrent
                          ? "#fff"
                          : "var(--accent-light)"
                        : "#6c6671",
                    }}
                  >
                    {icons[stage] || "circle"}
                  </i>
                </div>
                <span
                  style={{
                    fontSize: isCurrent ? 11.5 : 11,
                    color: isCurrent
                      ? "var(--text-primary)"
                      : isActive
                        ? "#9a949f"
                        : "#6c6671",
                    fontWeight: isCurrent ? 700 : 400,
                  }}
                >
                  {stage}
                </span>
              </div>
              {i < stages.length - 1 && (
                <div
                  style={{
                    flex: 1,
                    height: 2,
                    background: isActive ? "var(--accent)" : "#26222d",
                    marginBottom: 18,
                  }}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function HandoffCTA({ spec }: { spec: Spec }) {
  const handleCopy = async () => {
    const prompt = buildHandoffPrompt(spec.recommended_agent, spec.id, spec.status);
    try {
      await navigator.clipboard.writeText(prompt);
    } catch {
      // Fallback
      console.log("Copy:", prompt);
    }
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 16,
        background:
          "linear-gradient(100deg,rgba(124,108,246,.13),rgba(124,108,246,.05))",
        border: "1px solid rgba(124,108,246,.32)",
        borderRadius: 13,
        padding: "16px 18px",
        marginBottom: 26,
      }}
    >
      <div
        style={{
          width: 40,
          height: 40,
          borderRadius: 11,
          background: "rgba(124,108,246,.16)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flex: "none",
        }}
      >
        <i
          className="material-symbols-outlined"
          style={{ fontSize: 22, color: "var(--accent-light)" }}
        >
          pan_tool
        </i>
      </div>
      <div style={{ flex: 1 }}>
        <div
          style={{
            fontSize: 14.5,
            fontWeight: 700,
            marginBottom: 2,
          }}
        >
          Manual handoff required
        </div>
        <div
          style={{
            fontSize: 12.5,
            color: "#b6b1bb",
            lineHeight: 1.5,
          }}
        >
          Copy the prompt below, then{" "}
          <span style={{ color: "var(--text-primary)", fontWeight: 600 }}>
            start the {spec.recommended_agent || "specialist"} agent yourself
          </span>
          . LMBrain will not launch it for you.
        </div>
        <div
          style={{
            fontSize: 11.5,
            color: "#7fa8f5",
            marginTop: 6,
            lineHeight: 1.4,
          }}
        >
          <i
            className="material-symbols-outlined"
            style={{ fontSize: 13, verticalAlign: "middle", marginRight: 4 }}
          >
            lightbulb
          </i>
          The prompt now includes v3 context-economy guidance. The agent will use{" "}
          <span style={{ fontFamily: "var(--font-mono)", fontSize: 11 }}>
            lmbrain_spec_context
          </span>{" "}
          for a compact handoff context before expanding to full artifacts.
        </div>
      </div>
      <button
        onClick={handleCopy}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          background: "linear-gradient(180deg,#8676f7,#6e5bf2)",
          border: "none",
          color: "#fff",
          borderRadius: 9,
          padding: "11px 17px",
          fontSize: 13,
          fontWeight: 600,
          cursor: "pointer",
          whiteSpace: "nowrap",
          boxShadow: "0 8px 20px -7px rgba(110,91,242,.7)",
        }}
      >
        <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
          content_copy
        </i>
        Copy handoff prompt
      </button>
    </div>
  );
}
