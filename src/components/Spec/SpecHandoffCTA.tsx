import { buildHandoffPrompt } from "../../lib/handoffPrompt";
import type { Spec } from "../../types";

export interface SpecHandoffCTAProps {
  spec: Spec;
}

export function SpecHandoffCTA({ spec }: SpecHandoffCTAProps) {
  const handleCopy = async () => {
    const prompt = buildHandoffPrompt(spec.recommended_agent, spec.id, spec.status);
    try {
      await navigator.clipboard.writeText(prompt);
    } catch {
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
            fontSize: "var(--text-md)",
            fontWeight: 700,
            marginBottom: 2,
          }}
        >
          Manual handoff required
        </div>
        <div
          style={{
            fontSize: "var(--text-sm)",
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
            fontSize: "var(--text-xs)",
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
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)" }}>
            lmbrain_spec_context
          </span>{" "}
          for a compact handoff context before expanding to full artifacts.
        </div>
      </div>
      <button
        type="button"
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
          fontSize: "var(--text-md)",
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
