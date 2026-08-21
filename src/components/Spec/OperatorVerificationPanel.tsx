import { useEffect, useMemo, useState } from "react";
import type { CSSProperties, ReactNode } from "react";
import { getSpecVerification } from "../../lib/commands";
import { useWorkspace } from "../../hooks/useWorkspace";
import type {
  Spec,
  SpecVerificationState,
  VerificationRequirement,
} from "../../types";

interface OperatorVerificationPanelProps {
  spec: Spec;
}

export function OperatorVerificationPanel({ spec }: OperatorVerificationPanelProps) {
  const { navigateTo } = useWorkspace();
  const [verification, setVerification] = useState<SpecVerificationState | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadVerification = async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await getSpecVerification(spec.path);
      setVerification(next);
    } catch (loadError) {
      setError(errorMessage(loadError));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let active = true;
    getSpecVerification(spec.path)
      .then((next) => {
        if (!active) return;
        setVerification(next);
      })
      .catch((loadError: unknown) => {
        if (active) setError(errorMessage(loadError));
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [spec.path]);

  const requirements = useMemo(
    () =>
      verification?.requirements.filter(
        (requirement) => requirement.phase === "before-done",
      ) ?? [],
    [verification],
  );

  const hasOperatorGates = requirements.some(isOperatorGate);

  if (loading) {
    return <PanelShell>Loading before-done verification…</PanelShell>;
  }
  if (error && !verification) {
    return (
      <PanelShell>
        <InlineError message={error} />
        <RetryButton onClick={() => void loadVerification()} />
      </PanelShell>
    );
  }
  if (requirements.length === 0) {
    return null;
  }

  return (
    <PanelShell>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 14 }}>
        <div>
          <div style={{ fontSize: "var(--text-sm)", fontWeight: 700, color: "var(--text-primary)" }}>
            Verification gates
          </div>
          <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", marginTop: 2 }}>
            Requirements that must be satisfied before this spec can be marked done.
          </div>
        </div>
        {hasOperatorGates && (
          <button
            type="button"
            onClick={() => navigateTo("operations")}
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: 6,
              padding: "6px 12px",
              borderRadius: 6,
              fontSize: "var(--text-xs)",
              fontWeight: 600,
              background: "rgba(122, 162, 247, 0.12)",
              color: "var(--accent-primary, #7aa2f7)",
              border: "1px solid rgba(122, 162, 247, 0.3)",
              cursor: "pointer",
            }}
          >
            <span>Go to Operations</span>
            <i className="material-symbols-outlined" style={{ fontSize: 14 }}>
              arrow_forward
            </i>
          </button>
        )}
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        {requirements.map((requirement) => {
          const blocker = verification?.blockers.find(
            (b) => b.requirement_id === requirement.id,
          );
          const attestation = verification?.attestations.find(
            (a) => a.requirement_id === requirement.id && a.result === "passed",
          );
          const isAttested = !blocker && !!attestation;
          const isBlocked = !!blocker;

          return (
            <div
              key={requirement.id}
              style={{
                padding: "8px 12px",
                borderRadius: 6,
                background: "var(--bg-primary)",
                border: "1px solid var(--border-primary)",
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                gap: 12,
              }}
            >
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <span
                    style={{
                      fontFamily: "var(--font-mono, monospace)",
                      fontSize: "var(--text-xs)",
                      fontWeight: 700,
                      color: "var(--text-primary)",
                    }}
                  >
                    {requirement.id}
                  </span>
                  <span
                    style={{
                      fontSize: "var(--text-2xs)",
                      padding: "1px 5px",
                      borderRadius: 4,
                      textTransform: "uppercase",
                      fontWeight: 600,
                      background: "var(--bg-tertiary)",
                      color: "var(--text-secondary)",
                    }}
                  >
                    {requirement.owner}
                  </span>
                  <span style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
                    {requirement.text}
                  </span>
                </div>
                {blocker && (
                  <div style={{ fontSize: "var(--text-xs)", color: "var(--status-error, #f7768e)", marginTop: 4 }}>
                    Blocker: {blocker.cause}
                  </div>
                )}
                {attestation && (
                  <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)", marginTop: 4 }}>
                    Attested by {attestation.actor} ({attestation.timestamp}) — {attestation.evidence_ref}
                  </div>
                )}
              </div>

              <div>
                {isAttested && (
                  <span
                    style={{
                      fontSize: "var(--text-xs)",
                      padding: "2px 6px",
                      borderRadius: 4,
                      background: "rgba(158, 206, 106, 0.15)",
                      color: "#9ece6a",
                      fontWeight: 700,
                    }}
                  >
                    ATTESTED
                  </span>
                )}
                {isBlocked && (
                  <span
                    style={{
                      fontSize: "var(--text-xs)",
                      padding: "2px 6px",
                      borderRadius: 4,
                      background: "rgba(247, 118, 142, 0.15)",
                      color: "#f7768e",
                      fontWeight: 700,
                    }}
                  >
                    BLOCKED
                  </span>
                )}
                {!isAttested && !isBlocked && (
                  <span
                    style={{
                      fontSize: "var(--text-xs)",
                      padding: "2px 6px",
                      borderRadius: 4,
                      background: "rgba(224, 162, 58, 0.15)",
                      color: "#d9b86d",
                      fontWeight: 700,
                    }}
                  >
                    PENDING
                  </span>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </PanelShell>
  );
}

function isOperatorGate(requirement: VerificationRequirement): boolean {
  return requirement.owner === "operator";
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  return "An unknown error occurred";
}

function PanelShell({ children, style }: { children: ReactNode; style?: CSSProperties }) {
  return (
    <section
      aria-label="Spec verification gates"
      style={{
        marginBottom: 18,
        padding: "12px 14px",
        border: "1px solid var(--border-primary)",
        borderRadius: 8,
        background: "var(--bg-tertiary)",
        ...style,
      }}
    >
      {children}
    </section>
  );
}

function InlineError({ message }: { message: string }) {
  return (
    <div
      role="alert"
      style={{
        fontSize: "var(--text-xs)",
        color: "var(--status-error, #f7768e)",
        marginBottom: 8,
      }}
    >
      {message}
    </div>
  );
}

function RetryButton({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      style={{
        padding: "4px 8px",
        fontSize: "var(--text-xs)",
        borderRadius: 4,
        border: "1px solid var(--border-primary)",
        background: "var(--bg-secondary)",
        color: "var(--text-primary)",
        cursor: "pointer",
      }}
    >
      Retry
    </button>
  );
}
