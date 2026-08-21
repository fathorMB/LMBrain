import { useCallback, useMemo, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { useDialog } from "../../hooks/useDialog";
import { attestOperatorVerification } from "../../lib/commands";
import type { OperatorGate } from "../../types";
import { PageHeader, PageShell } from "../Shared/PageLayout";
import { FilterBar, FilterSearchInput, FilterSelect } from "../Shared/FilterBar";
import { RefreshButton } from "../RefreshButton";
import { ModalCloseButton } from "../Layout/ModalCloseButton";

const OPERATOR_IDENTITY_KEY = "lmbrain.operator.identity";

export function OperationsView() {
  const { state, refreshWorkspaceData, openSpec } = useWorkspace();
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [milestoneFilter, setMilestoneFilter] = useState<string>("all");
  const [gateStateFilter, setGateStateFilter] = useState<string>("all");
  const [query, setQuery] = useState<string>("");
  const [selectedGate, setSelectedGate] = useState<OperatorGate | null>(null);

  const closeAttestModal = useCallback(() => setSelectedGate(null), []);

  const gates = useMemo(() => {
    return (state.operatorGates ?? []).filter((gate) => {
      if (statusFilter !== "all" && gate.spec_status !== statusFilter) {
        return false;
      }
      if (milestoneFilter !== "all" && (gate.milestone ?? "None") !== milestoneFilter) {
        return false;
      }
      if (gateStateFilter === "pending" && (gate.attested !== null || gate.blocker !== null)) {
        return false;
      }
      if (gateStateFilter === "attested" && gate.attested === null) {
        return false;
      }
      if (gateStateFilter === "blocked" && gate.blocker === null) {
        return false;
      }
      if (query.trim()) {
        const q = query.trim().toLowerCase();
        const text = `${gate.spec_id} ${gate.spec_title} ${gate.requirement_id} ${gate.text} ${gate.evidence_kind} ${gate.milestone ?? ""}`.toLowerCase();
        if (!text.includes(q)) {
          return false;
        }
      }
      return true;
    });
  }, [state.operatorGates, statusFilter, milestoneFilter, gateStateFilter, query]);

  const milestones = useMemo(() => {
    const set = new Set<string>();
    for (const gate of state.operatorGates ?? []) {
      set.add(gate.milestone ?? "None");
    }
    return Array.from(set).sort();
  }, [state.operatorGates]);

  const pendingCount = useMemo(() => {
    return (state.operatorGates ?? []).filter((g) => g.attested === null || g.blocker !== null).length;
  }, [state.operatorGates]);

  return (
    <PageShell archetype="dense">
      <PageHeader
        title="Operations"
        description="Active operator verification gates across specs in review and done. Record evidence attestations to clear verification gates."
        actions={
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            {pendingCount > 0 && (
              <span
                style={{
                  fontSize: "var(--text-xs)",
                  padding: "4px 8px",
                  borderRadius: 12,
                  background: "rgba(224, 162, 58, 0.15)",
                  color: "#d9b86d",
                  fontWeight: 600,
                  border: "1px solid rgba(224, 162, 58, 0.3)",
                }}
              >
                {pendingCount} pending
              </span>
            )}
            <RefreshButton loading={state.dataRefreshing} onClick={refreshWorkspaceData} />
          </div>
        }
      />

      <FilterBar>
        <FilterSelect
          label="Spec Status"
          value={statusFilter}
          onChange={setStatusFilter}
          options={[
            { label: "All Spec Statuses", value: "all" },
            { label: "Review", value: "review" },
            { label: "Done", value: "done" },
          ]}
        />
        <FilterSelect
          label="Milestone"
          value={milestoneFilter}
          onChange={setMilestoneFilter}
          options={[
            { label: "All Milestones", value: "all" },
            ...milestones.map((m) => ({ label: m, value: m })),
          ]}
        />
        <FilterSelect
          label="Gate State"
          value={gateStateFilter}
          onChange={setGateStateFilter}
          options={[
            { label: "All Gate States", value: "all" },
            { label: "Pending", value: "pending" },
            { label: "Attested", value: "attested" },
            { label: "Blocked", value: "blocked" },
          ]}
        />
        <FilterSearchInput
          value={query}
          onChange={setQuery}
          placeholder="Filter by spec, requirement, or evidence..."
        />
      </FilterBar>

      {gates.length === 0 ? (
        <div
          style={{
            padding: "48px 24px",
            textAlign: "center",
            color: "var(--text-tertiary)",
            background: "var(--bg-secondary)",
            borderRadius: 8,
            border: "1px solid var(--border-primary)",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 36, opacity: 0.5, marginBottom: 8 }}>
            verified_user
          </i>
          <div style={{ fontSize: "var(--text-md)", fontWeight: 600, color: "var(--text-secondary)" }}>
            No operator verification gates match the selected filters
          </div>
          <div style={{ fontSize: "var(--text-sm)", marginTop: 4 }}>
            {state.operatorGates.length === 0
              ? "All specs have satisfied operator gates or no operator verification is required."
              : "Try adjusting your filters above."}
          </div>
        </div>
      ) : (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 8,
          }}
        >
          {gates.map((gate) => {
            const isAttested = gate.attested !== null && gate.blocker === null;
            const isBlocked = gate.blocker !== null;
            const isPending = gate.attested === null && gate.blocker === null;

            return (
              <div
                key={`${gate.spec_id}-${gate.requirement_id}`}
                style={{
                  background: "var(--bg-secondary)",
                  border: "1px solid var(--border-primary)",
                  borderRadius: 8,
                  padding: "12px 16px",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  gap: 16,
                }}
              >
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap", marginBottom: 4 }}>
                    <button
                      type="button"
                      onClick={() => {
                        const target = state.specs.find((s) => s.id === gate.spec_id);
                        if (target) openSpec(target);
                      }}
                      style={{
                        background: "none",
                        border: "none",
                        padding: 0,
                        fontFamily: "var(--font-mono, monospace)",
                        fontWeight: 700,
                        fontSize: "var(--text-sm)",
                        color: "var(--accent-primary, #9d7cd8)",
                        cursor: "pointer",
                        textDecoration: "underline",
                      }}
                    >
                      {gate.spec_id}
                    </button>
                    <span
                      style={{
                        fontSize: "var(--text-2xs)",
                        padding: "1px 6px",
                        borderRadius: 4,
                        textTransform: "uppercase",
                        fontWeight: 700,
                        background:
                          gate.spec_status === "review"
                            ? "rgba(187, 154, 247, 0.15)"
                            : "rgba(158, 206, 106, 0.15)",
                        color:
                          gate.spec_status === "review" ? "#bb9af7" : "#9ece6a",
                      }}
                    >
                      {gate.spec_status}
                    </span>
                    {gate.milestone && (
                      <span
                        style={{
                          fontSize: "var(--text-2xs)",
                          padding: "1px 6px",
                          borderRadius: 4,
                          background: "var(--bg-tertiary)",
                          color: "var(--text-tertiary)",
                        }}
                      >
                        {gate.milestone}
                      </span>
                    )}
                    <span style={{ fontSize: "var(--text-sm)", fontWeight: 600, color: "var(--text-primary)" }}>
                      {gate.spec_title}
                    </span>
                  </div>

                  <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 4 }}>
                    <span
                      style={{
                        fontFamily: "var(--font-mono, monospace)",
                        fontSize: "var(--text-xs)",
                        fontWeight: 600,
                        color: "var(--text-secondary)",
                      }}
                    >
                      {gate.requirement_id}
                    </span>
                    <span style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)" }}>
                      {gate.text}
                    </span>
                  </div>

                  <div style={{ display: "flex", alignItems: "center", gap: 12, marginTop: 6, fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>
                    <span>Evidence: <strong>{gate.evidence_kind}</strong></span>
                    {gate.attested && (
                      <span>
                        Attested by <strong>{gate.attested.actor}</strong> ({gate.attested.timestamp})
                        {gate.attested.delegated_by ? ` (delegated via ${gate.attested.delegation_channel ?? "channel"})` : ""}
                      </span>
                    )}
                    {isBlocked && (
                      <span style={{ color: "var(--status-error, #f7768e)" }}>
                        Blocked: {gate.blocker}
                      </span>
                    )}
                  </div>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: 12, flexShrink: 0 }}>
                  {isAttested && (
                    <span
                      style={{
                        fontSize: "var(--text-xs)",
                        padding: "3px 8px",
                        borderRadius: 6,
                        background: "rgba(158, 206, 106, 0.15)",
                        color: "#9ece6a",
                        fontWeight: 700,
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 4,
                      }}
                    >
                      <i className="material-symbols-outlined" style={{ fontSize: 14 }}>
                        check_circle
                      </i>
                      ATTESTED
                    </span>
                  )}
                  {isBlocked && (
                    <span
                      style={{
                        fontSize: "var(--text-xs)",
                        padding: "3px 8px",
                        borderRadius: 6,
                        background: "rgba(247, 118, 142, 0.15)",
                        color: "#f7768e",
                        fontWeight: 700,
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 4,
                      }}
                    >
                      <i className="material-symbols-outlined" style={{ fontSize: 14 }}>
                        cancel
                      </i>
                      BLOCKED
                    </span>
                  )}
                  {isPending && (
                    <span
                      style={{
                        fontSize: "var(--text-xs)",
                        padding: "3px 8px",
                        borderRadius: 6,
                        background: "rgba(224, 162, 58, 0.15)",
                        color: "#d9b86d",
                        fontWeight: 700,
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 4,
                      }}
                    >
                      <i className="material-symbols-outlined" style={{ fontSize: 14 }}>
                        pending
                      </i>
                      PENDING
                    </span>
                  )}

                  <button
                    type="button"
                    onClick={() => setSelectedGate(gate)}
                    style={{
                      padding: "6px 12px",
                      borderRadius: 6,
                      fontSize: "var(--text-xs)",
                      fontWeight: 600,
                      background: "var(--bg-tertiary)",
                      color: "var(--text-primary)",
                      border: "1px solid var(--border-primary)",
                      cursor: "pointer",
                    }}
                  >
                    {isAttested ? "Re-Attest / View" : "Attest Gate"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {selectedGate && (
        <AttestModal
          gate={selectedGate}
          onClose={closeAttestModal}
          onSuccess={async () => {
            closeAttestModal();
            await refreshWorkspaceData();
          }}
        />
      )}
    </PageShell>
  );
}

interface AttestModalProps {
  gate: OperatorGate;
  onClose: () => void;
  onSuccess: () => Promise<void>;
}

function AttestModal({ gate, onClose, onSuccess }: AttestModalProps) {
  const { openSpec, state } = useWorkspace();
  const [actor, setActor] = useState<string>(() => {
    return localStorage.getItem(OPERATOR_IDENTITY_KEY) ?? "";
  });
  const [evidenceRef, setEvidenceRef] = useState<string>("");
  const [submitting, setSubmitting] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const { dialogRef, handleKeyDown } = useDialog({
    isOpen: true,
    onClose,
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!actor.trim()) {
      setError("Operator identity is required.");
      return;
    }
    if (!evidenceRef.trim()) {
      setError("Evidence reference is required.");
      return;
    }

    setSubmitting(true);
    setError(null);
    try {
      localStorage.setItem(OPERATOR_IDENTITY_KEY, actor.trim());
      await attestOperatorVerification(
        gate.spec_path,
        gate.requirement_id,
        actor.trim(),
        evidenceRef.trim(),
      );
      await onSuccess();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setSubmitting(false);
    }
  };

  return (
    <div
      role="presentation"
      onKeyDown={handleKeyDown}
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      style={{
        position: "fixed",
        inset: 0,
        backgroundColor: "rgba(0, 0, 0, 0.75)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 1000,
      }}
    >
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions */}
      <div
        ref={dialogRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="attest-modal-title"
        tabIndex={-1}
        onMouseDown={(e) => e.stopPropagation()}
        style={{
          width: "100%",
          maxWidth: 540,
          background: "var(--bg-secondary)",
          border: "1px solid var(--border-primary)",
          borderRadius: 8,
          boxShadow: "0 8px 32px rgba(0, 0, 0, 0.5)",
          padding: 24,
          position: "relative",
          outline: "none",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 16 }}>
          <h2 id="attest-modal-title" style={{ fontSize: "var(--text-lg)", fontWeight: 700, margin: 0 }}>
            Attest Operator Verification
          </h2>
          <ModalCloseButton onClick={onClose} label="Close modal" />
        </div>

        <div style={{ marginBottom: 16, padding: "10px 12px", background: "var(--bg-tertiary)", borderRadius: 6, fontSize: "var(--text-sm)" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 4 }}>
            <button
              type="button"
              onClick={() => {
                const target = state.specs.find((s) => s.id === gate.spec_id);
                if (target) {
                  onClose();
                  openSpec(target);
                }
              }}
              style={{
                background: "none",
                border: "none",
                padding: 0,
                fontFamily: "var(--font-mono, monospace)",
                fontWeight: 700,
                color: "var(--accent-primary, #9d7cd8)",
                cursor: "pointer",
                textDecoration: "underline",
              }}
            >
              {gate.spec_id}
            </button>
            <span style={{ fontWeight: 600, color: "var(--text-primary)" }}>{gate.spec_title}</span>
          </div>
          <div style={{ color: "var(--text-secondary)", marginTop: 6 }}>
            <strong>{gate.requirement_id}:</strong> {gate.text}
          </div>
          <div style={{ color: "var(--text-tertiary)", fontSize: "var(--text-xs)", marginTop: 4 }}>
            Required evidence: {gate.evidence_kind} · Phase: before-done
          </div>
        </div>

        {gate.blocker && (
          <div
            role="alert"
            style={{
              padding: "8px 12px",
              marginBottom: 16,
              borderRadius: 6,
              background: "rgba(247, 118, 142, 0.15)",
              border: "1px solid rgba(247, 118, 142, 0.3)",
              color: "#f7768e",
              fontSize: "var(--text-xs)",
              display: "flex",
              alignItems: "center",
              gap: 8,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 16 }}>
              warning
            </i>
            <span>Blocked: {gate.blocker}</span>
          </div>
        )}

        {gate.attested && (
          <div
            style={{
              padding: "8px 12px",
              marginBottom: 16,
              borderRadius: 6,
              background: "rgba(158, 206, 106, 0.1)",
              border: "1px solid rgba(158, 206, 106, 0.25)",
              color: "#9ece6a",
              fontSize: "var(--text-xs)",
            }}
          >
            <div>Previously attested by <strong>{gate.attested.actor}</strong> on {gate.attested.timestamp}</div>
            <div style={{ color: "var(--text-tertiary)", marginTop: 2 }}>Evidence: {gate.attested.evidence_ref}</div>
          </div>
        )}

        {error && (
          <div
            role="alert"
            style={{
              padding: "8px 12px",
              marginBottom: 16,
              borderRadius: 6,
              background: "rgba(247, 118, 142, 0.15)",
              color: "#f7768e",
              fontSize: "var(--text-xs)",
            }}
          >
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: 14 }}>
          <div>
            <label
              htmlFor="operator-identity-input"
              style={{ display: "block", fontSize: "var(--text-xs)", fontWeight: 600, marginBottom: 4, color: "var(--text-secondary)" }}
            >
              Operator Identity
            </label>
            <input
              id="operator-identity-input"
              type="text"
              value={actor}
              onChange={(e) => setActor(e.target.value)}
              placeholder="e.g. operator@local / Operator Name"
              required
              style={{
                width: "100%",
                padding: "8px 10px",
                borderRadius: 6,
                background: "var(--bg-primary)",
                border: "1px solid var(--border-primary)",
                color: "var(--text-primary)",
                fontSize: "var(--text-sm)",
                outline: "none",
                boxSizing: "border-box",
              }}
            />
          </div>

          <div>
            <label
              htmlFor="operator-evidence-input"
              style={{ display: "block", fontSize: "var(--text-xs)", fontWeight: 600, marginBottom: 4, color: "var(--text-secondary)" }}
            >
              Evidence Reference
            </label>
            <textarea
              id="operator-evidence-input"
              rows={3}
              value={evidenceRef}
              onChange={(e) => setEvidenceRef(e.target.value)}
              placeholder="Describe observations, test runs, screenshots, or local PR evidence..."
              required
              style={{
                width: "100%",
                padding: "8px 10px",
                borderRadius: 6,
                background: "var(--bg-primary)",
                border: "1px solid var(--border-primary)",
                color: "var(--text-primary)",
                fontSize: "var(--text-sm)",
                outline: "none",
                resize: "vertical",
                boxSizing: "border-box",
              }}
            />
          </div>

          <div style={{ display: "flex", justifyContent: "flex-end", gap: 10, marginTop: 8 }}>
            <button
              type="button"
              onClick={onClose}
              style={{
                padding: "8px 14px",
                borderRadius: 6,
                fontSize: "var(--text-sm)",
                fontWeight: 600,
                background: "transparent",
                border: "1px solid var(--border-primary)",
                color: "var(--text-secondary)",
                cursor: "pointer",
              }}
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={submitting}
              style={{
                padding: "8px 16px",
                borderRadius: 6,
                fontSize: "var(--text-sm)",
                fontWeight: 600,
                background: "var(--accent-primary, #7aa2f7)",
                border: "none",
                color: "#1a1b26",
                cursor: submitting ? "not-allowed" : "pointer",
                opacity: submitting ? 0.7 : 1,
              }}
            >
              {submitting ? "Recording..." : "Record Attestation"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
