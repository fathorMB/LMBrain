import { useEffect, useMemo, useState } from "react";
import type { CSSProperties, ReactNode } from "react";
import {
  attestOperatorVerification,
  getSpecVerification,
} from "../../lib/commands";
import type {
  Spec,
  SpecVerificationState,
  VerificationRequirement,
} from "../../types";

interface OperatorVerificationPanelProps {
  spec: Spec;
  onAttested: () => Promise<void>;
}

export function OperatorVerificationPanel({
  spec,
  onAttested,
}: OperatorVerificationPanelProps) {
  const [verification, setVerification] =
    useState<SpecVerificationState | null>(null);
  const [selectedId, setSelectedId] = useState("");
  const [actor, setActor] = useState("");
  const [evidenceRef, setEvidenceRef] = useState("");
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadVerification = async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await getSpecVerification(spec.path);
      setVerification(next);
      const operatorRequirements = next.requirements.filter(isOperatorGate);
      setSelectedId((current) =>
        operatorRequirements.some((requirement) => requirement.id === current)
          ? current
          : operatorRequirements[0]?.id ?? "",
      );
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
        if (!active) {
          return;
        }
        setVerification(next);
        const operatorRequirements = next.requirements.filter(isOperatorGate);
        setSelectedId(operatorRequirements[0]?.id ?? "");
      })
      .catch((loadError: unknown) => {
        if (active) {
          setError(errorMessage(loadError));
        }
      })
      .finally(() => {
        if (active) {
          setLoading(false);
        }
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
  const operatorRequirements = requirements.filter(isOperatorGate);
  const selected = operatorRequirements.find(
    (requirement) => requirement.id === selectedId,
  );


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

  const submit = async () => {
    if (!selected || !actor.trim() || !evidenceRef.trim()) {
      return;
    }
    setSubmitting(true);
    setError(null);
    try {
      await attestOperatorVerification(
        spec.path,
        selected.id,
        actor.trim(),
        evidenceRef.trim(),
      );
      setEvidenceRef("");
      await onAttested();
      await loadVerification();
    } catch (submitError) {
      setError(errorMessage(submitError));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <PanelShell>
      <div style={{ marginBottom: 14 }}>
        <div
          style={{
            fontSize: 13.5,
            fontWeight: 700,
            color: "var(--text-primary)",
            marginBottom: 4,
          }}
        >
          Before-done verification
        </div>
        <div style={{ fontSize: 12, color: "#9a949f", lineHeight: 1.45 }}>
          Evidence attestation only — this does not approve the spec or change
          its status.
        </div>
      </div>

      <div style={{ display: "grid", gap: 8 }}>
        {requirements.map((requirement) => {
          const blocker = verification?.blockers.find(
            (candidate) => candidate.requirement_id === requirement.id,
          );
          const attestation = verification?.attestations
            .slice()
            .reverse()
            .find(
              (candidate) =>
                candidate.requirement_id === requirement.id &&
                candidate.actor_role === requirement.owner,
            );
          return (
            <div
              key={`${requirement.owner}:${requirement.id}`}
              style={{
                border: "1px solid #2a2631",
                background: "#121016",
                borderRadius: 9,
                padding: "10px 12px",
              }}
            >
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 8,
                  flexWrap: "wrap",
                }}
              >
                <code style={{ color: "#bcaef6", fontSize: 11.5 }}>
                  {requirement.id}
                </code>
                <span style={{ fontSize: 11, color: "#807986" }}>
                  owner={requirement.owner}
                </span>
                <span
                  style={{
                    fontSize: 10.5,
                    color: blocker ? "#f0a2a2" : "#91d5ad",
                  }}
                >
                  {blocker ? "BLOCKED" : "ATTESTED"}
                </span>
              </div>
              <div style={{ fontSize: 12.5, marginTop: 5 }}>
                {requirement.text}
              </div>
              <div
                style={{
                  fontSize: 11,
                  color: blocker ? "#c88f8f" : "#77717d",
                  marginTop: 5,
                }}
              >
                {blocker
                  ? blocker.cause
                  : attestation
                    ? `Evidence: ${attestation.evidence_ref} · ${attestation.actor}`
                    : "Verification requirement satisfied"}
              </div>
            </div>
          );
        })}
      </div>

      {operatorRequirements.length > 0 && (
        <div
          style={{
            marginTop: 14,
            paddingTop: 14,
            borderTop: "1px solid #292530",
          }}
        >
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "minmax(140px,.7fr) minmax(180px,1fr)",
              gap: 10,
            }}
          >
            <Field label="Operator gate">
              <select
                aria-label="Operator gate"
                value={selectedId}
                onChange={(event) => setSelectedId(event.target.value)}
                style={fieldStyle}
              >
                {operatorRequirements.map((requirement) => (
                  <option key={requirement.id} value={requirement.id}>
                    {requirement.id}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Attestor identity">
              <input
                aria-label="Attestor identity"
                value={actor}
                onChange={(event) => setActor(event.target.value)}
                placeholder="Your name or operator ID"
                style={fieldStyle}
              />
            </Field>
          </div>
          <div style={{ marginTop: 10 }}>
            <Field label="Evidence reference">
              <input
                aria-label="Evidence reference"
                value={evidenceRef}
                onChange={(event) => setEvidenceRef(event.target.value)}
                placeholder="Artifact ID, workspace path, or observation reference"
                style={fieldStyle}
              />
            </Field>
          </div>
          {error && <InlineError message={error} />}
          {selected && !selected.checked && (
            <div
              role="status"
              style={{ color: "#9aadcf", fontSize: 11.5, marginTop: 10 }}
            >
              This checklist item will be marked complete when you attest.
            </div>
          )}
          <button
            type="button"
            onClick={() => void submit()}
            disabled={
              submitting ||
              !selected ||
              !actor.trim() ||
              !evidenceRef.trim()
            }
            style={{
              marginTop: 12,
              border: "1px solid rgba(124,108,246,.45)",
              borderRadius: 8,
              background: "rgba(124,108,246,.14)",
              color: "var(--accent-light)",
              padding: "9px 13px",
              fontSize: 12,
              fontWeight: 650,
              cursor: submitting ? "wait" : "pointer",
              opacity:
                submitting ||
                !selected ||
                !actor.trim() ||
                !evidenceRef.trim()
                  ? 0.55
                  : 1,
            }}
          >
            {submitting ? "Recording evidence…" : "Attest evidence"}
          </button>
        </div>
      )}
    </PanelShell>
  );
}

function isOperatorGate(requirement: VerificationRequirement) {
  return requirement.owner === "operator" && requirement.phase === "before-done";
}

function PanelShell({ children }: { children: ReactNode }) {
  return (
    <section
      aria-label="Before-done verification"
      style={{
        background: "#100e14",
        border: "1px solid #27222e",
        borderRadius: 12,
        padding: "16px 18px",
        marginBottom: 20,
      }}
    >
      {children}
    </section>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <label style={{ display: "grid", gap: 5, fontSize: 11, color: "#8f8994" }}>
      {label}
      {children}
    </label>
  );
}

function InlineError({ message }: { message: string }) {
  return (
    <div role="alert" style={{ color: "#efaaaa", fontSize: 11.5, marginTop: 10 }}>
      {message}
    </div>
  );
}

function RetryButton({ onClick }: { onClick: () => void }) {
  return (
    <button type="button" onClick={onClick} style={{ marginTop: 10 }}>
      Retry
    </button>
  );
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

const fieldStyle: CSSProperties = {
  width: "100%",
  boxSizing: "border-box",
  background: "#15121a",
  color: "var(--text-primary)",
  border: "1px solid #332e3a",
  borderRadius: 7,
  padding: "8px 9px",
  fontSize: 12,
};
