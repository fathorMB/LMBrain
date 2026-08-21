import { useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { buildHandoffPrompt } from "../../lib/handoffPrompt";
import { InlineRichText } from "../../lib/inlineRichText";
import { useWikiNavigation } from "../../hooks/useWikiNavigation";
import type { PulseData } from "../../types";

export interface PulseActionCardProps {
  action: PulseData["actions"][0];
}

export function PulseActionCard({ action }: PulseActionCardProps) {
  const { state } = useWorkspace();
  const [expanded, setExpanded] = useState(false);
  const [copyState, setCopyState] = useState<"idle" | "copied" | "error">("idle");
  const navigateToWiki = useWikiNavigation();
  const isHandoff = action.action_type === "handoff" && action.spec_id;

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
