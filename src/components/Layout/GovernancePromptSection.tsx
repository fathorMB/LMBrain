import { useState } from "react";
import {
  generateRejectedPrompt,
  generateSpecApprovalPrompt,
  generateAgentActivationPrompt,
  generateAdrDecisionPrompt,
} from "../../lib/governancePrompts";

export function GovernancePromptCard({ prompt }: { prompt: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <div style={{ position: "relative" }}>
      <textarea
        readOnly
        value={prompt}
        onClick={(e) => e.currentTarget.select()}
        style={{
          width: "100%",
          height: 120,
          background: "var(--bg-primary)",
          border: "1px solid var(--border-primary)",
          borderRadius: 6,
          padding: "8px 12px",
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-xs)",
          color: "var(--text-secondary)",
          resize: "none",
          outline: "none",
        }}
      />
      <button
        type="button"
        onClick={() => {
          navigator.clipboard?.writeText(prompt);
          setCopied(true);
          setTimeout(() => setCopied(false), 2000);
        }}
        style={{
          position: "absolute",
          right: 8,
          bottom: 12,
          background: "rgba(255,255,255,0.06)",
          border: "1px solid rgba(255,255,255,0.1)",
          borderRadius: 6,
          padding: "4px 10px",
          fontSize: "var(--text-xs)",
          color: "#fff",
          cursor: "pointer",
          fontWeight: 600,
        }}
      >
        {copied ? "Copied!" : "Copy prompt"}
      </button>
    </div>
  );
}

export interface GovernancePromptSectionProps {
  id: string;
  title: string;
  path: string;
  showGovernancePrompt: boolean;
  status: string | undefined;
}

export function GovernancePromptSection({
  id,
  title,
  path,
  showGovernancePrompt,
  status,
}: GovernancePromptSectionProps) {
  const [promptCopied, setPromptCopied] = useState(false);

  return (
    <>
      {status === "rejected" && id && (
        <div
          style={{
            marginTop: 24,
            padding: 16,
            background: "rgba(224, 88, 74, 0.08)",
            border: "1px solid rgba(224, 88, 74, 0.2)",
            borderRadius: 10,
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
            <i className="material-symbols-outlined" style={{ color: "#e0584a", fontSize: 20 }}>
              info
            </i>
            <span style={{ fontSize: "var(--text-md)", fontWeight: 600, color: "#fff" }}>
              Artifact Rejected
            </span>
          </div>
          <p style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)", margin: "0 0 12px" }}>
            This proposal was rejected. Copy the corrective prompt below to have an agent revise the file:
          </p>
          <div style={{ position: "relative" }}>
            <textarea
              readOnly
              value={generateRejectedPrompt(path, id)}
              style={{
                width: "100%",
                height: 120,
                background: "var(--bg-primary)",
                border: "1px solid var(--border-primary)",
                borderRadius: 6,
                padding: "8px 12px",
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-xs)",
                color: "var(--text-secondary)",
                resize: "none",
                outline: "none",
              }}
            />
            <button
              type="button"
              onClick={() => {
                navigator.clipboard?.writeText(generateRejectedPrompt(path, id));
                setPromptCopied(true);
                setTimeout(() => setPromptCopied(false), 2000);
              }}
              style={{
                position: "absolute",
                right: 8,
                bottom: 12,
                background: "rgba(255,255,255,0.06)",
                border: "1px solid rgba(255,255,255,0.1)",
                borderRadius: 6,
                padding: "4px 10px",
                fontSize: "var(--text-xs)",
                color: "#fff",
                cursor: "pointer",
                fontWeight: 600,
              }}
            >
              {promptCopied ? "Copied!" : "Copy prompt"}
            </button>
          </div>
        </div>
      )}

      {showGovernancePrompt && id && (
        <div
          style={{
            marginTop: 24,
            padding: 16,
            background: "rgba(91, 141, 239, 0.08)",
            border: "1px solid rgba(91, 141, 239, 0.2)",
            borderRadius: 10,
          }}
        >
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 12 }}>
            <i className="material-symbols-outlined" style={{ color: "#7fa8f5", fontSize: 20 }}>
              info
            </i>
            <span style={{ fontSize: "var(--text-md)", fontWeight: 600, color: "#fff" }}>
              {id.startsWith("SPEC-")
                ? "Spec Approval"
                : id.startsWith("ADR-")
                  ? "ADR Decision"
                  : "Agent Profile Activation"}
            </span>
          </div>
          <p style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)", margin: "0 0 12px" }}>
            {id.startsWith("SPEC-")
              ? "Spec approval is performed by the Project Lead on explicit operator instruction. Copy the prompt below and give it to the Project Lead."
              : id.startsWith("ADR-")
                ? "ADR acceptance or rejection is performed by the Project Lead on explicit operator instruction. Copy the intended decision prompt below and give it to the Project Lead."
                : "Agent profile activation is performed through the Project Lead workflow on explicit operator instruction. Copy the prompt below and give it to the Project Lead."}
          </p>
          {id.startsWith("ADR-") ? (
            <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
              <div>
                <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", marginBottom: 6 }}>
                  Accept decision prompt
                </div>
                <GovernancePromptCard prompt={generateAdrDecisionPrompt(id, title, path, "accepted")} />
              </div>
              <div>
                <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", marginBottom: 6 }}>
                  Reject decision prompt
                </div>
                <GovernancePromptCard prompt={generateAdrDecisionPrompt(id, title, path, "rejected")} />
              </div>
            </div>
          ) : (
            <GovernancePromptCard
              prompt={
                id.startsWith("SPEC-")
                  ? generateSpecApprovalPrompt(id, title, path)
                  : generateAgentActivationPrompt(id, title, path)
              }
            />
          )}
        </div>
      )}
    </>
  );
}
