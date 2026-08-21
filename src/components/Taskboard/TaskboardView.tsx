import { useEffect, useMemo, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getSpecs } from "../../lib/commands";
import {
  CAPABILITY_TIERS,
  EMPTY_BOARD_FILTERS,
  collectTagVocabulary,
  hasActiveBoardFilters,
  matchesBoardFilters,
  toggleValue,
  type BoardFilters,
} from "../../lib/boardFilters";
import type { Spec, SpecStatus } from "../../types";

const COLUMNS: { status: SpecStatus; label: string; color: string }[] = [
  { status: "backlog", label: "Backlog", color: "var(--text-tertiary)" },
  { status: "ready", label: "Ready", color: "#8a8d99" },
  { status: "working", label: "Working", color: "#5b8def" },
  { status: "review", label: "Review", color: "#e0a23a" },
  { status: "done", label: "Done", color: "#46b07d" },
  { status: "discarded", label: "Discarded", color: "#e0584a" },
];

function criteriaProgress(body: string): { done: number; total: number } {
  let done = 0;
  let total = 0;
  for (const line of body.split("\n")) {
    const t = line.trimStart();
    if (t.startsWith("- [x]") || t.startsWith("- [X]")) {
      done += 1;
      total += 1;
    } else if (t.startsWith("- [ ]")) {
      total += 1;
    }
  }
  return { done, total };
}

/** Tier colours track footprint: cool for small, warm for cross-layer. */
const TIER_COLORS: Record<string, string> = {
  luna: "#5b8def",
  terra: "#46b07d",
  sol: "#e0a23a",
};

const selectStyle = {
  background: "var(--bg-tertiary)",
  color: "var(--text-primary)",
  border: "1px solid var(--border-primary)",
  borderRadius: "var(--radius-sm)",
  padding: "5px 7px",
} as const;

const chipButtonStyle = {
  background: "var(--bg-tertiary)",
  color: "var(--text-secondary)",
  border: "1px solid var(--border-secondary)",
  borderRadius: "var(--radius-pill)",
  padding: "2px 9px",
  fontSize: "var(--text-xs)",
} as const;

export function TaskboardView() {
  const { state, openSpec } = useWorkspace();
  const [filters, setFilters] = useState<BoardFilters>(EMPTY_BOARD_FILTERS);

  const tagVocabulary = useMemo(() => collectTagVocabulary(state.specs), [state.specs]);
  const filtersActive = hasActiveBoardFilters(filters);

  const specsByStatus = (status: SpecStatus) =>
    state.specs.filter(
      (spec) => spec.status === status && matchesBoardFilters(spec, filters, state.specs),
    );
  const totalByStatus = (status: SpecStatus) =>
    state.specs.filter((spec) => spec.status === status).length;

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", minHeight: 0 }}>
      {/* Header */}
      <div
        style={{
          flex: "none",
          padding: "var(--space-5) var(--page-gutter) var(--space-4)",
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: 14,
          }}
        >
          <h1 style={{ fontSize: "var(--text-2xl)", fontWeight: 800, letterSpacing: "-.025em", margin: 0 }}>
            Board
          </h1>
          <div
            style={{
              fontSize: "var(--text-sm)",
              color: "var(--text-tertiary)",
              display: "flex",
              alignItems: "center",
              gap: 7,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 15, color: "var(--green)" }}>
              cloud_done
            </i>
            backed by{" "}
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "var(--text-secondary)" }}>
              .lmbrain/specs/&lt;status&gt;/*.md
            </span>
          </div>
        </div>
        <div
          style={{
            fontSize: "var(--text-xs)",
            color: "var(--text-muted)",
            display: "flex",
            alignItems: "center",
            gap: 10,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
            visibility
          </i>
          <label>
            Dependency view{" "}
            <select
              className="app-select"
              aria-label="Dependency view"
              value={filters.dependency}
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  dependency: event.target.value as BoardFilters["dependency"],
                }))
              }
              style={selectStyle}
            >
              <option value="all">All specs</option>
              <option value="blocked">Blocked by dependency</option>
              <option value="ready-after">Prerequisites complete</option>
            </select>
          </label>

          <label>
            Tier{" "}
            <select
              className="app-select"
              aria-label="Capability tier"
              value={filters.tiers[0] ?? "all"}
              onChange={(event) =>
                setFilters((current) => ({
                  ...current,
                  tiers: event.target.value === "all" ? [] : [event.target.value],
                }))
              }
              style={selectStyle}
            >
              <option value="all">Any tier</option>
              {CAPABILITY_TIERS.map((tier) => (
                <option key={tier} value={tier}>
                  {tier}
                </option>
              ))}
            </select>
          </label>

          {tagVocabulary.length > 0 && (
            <>
              <label>
                Tag{" "}
                <select
                  className="app-select"
                  aria-label="Add tag filter"
                  value=""
                  onChange={(event) => {
                    const value = event.target.value;
                    if (!value) return;
                    setFilters((current) => ({
                      ...current,
                      includeTags: toggleValue(current.includeTags, value),
                      excludeTags: current.excludeTags.filter((tag) => tag !== value),
                    }));
                  }}
                  style={selectStyle}
                >
                  <option value="">Include…</option>
                  {tagVocabulary.map((tag) => (
                    <option key={tag} value={tag}>
                      {tag}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                <select
                  className="app-select"
                  aria-label="Exclude tag filter"
                  value=""
                  onChange={(event) => {
                    const value = event.target.value;
                    if (!value) return;
                    setFilters((current) => ({
                      ...current,
                      excludeTags: toggleValue(current.excludeTags, value),
                      includeTags: current.includeTags.filter((tag) => tag !== value),
                    }));
                  }}
                  style={selectStyle}
                >
                  <option value="">Exclude…</option>
                  {tagVocabulary.map((tag) => (
                    <option key={tag} value={tag}>
                      {tag}
                    </option>
                  ))}
                </select>
              </label>
              {filters.includeTags.length > 1 && (
                <button
                  type="button"
                  aria-pressed={filters.includeMode === "all"}
                  onClick={() =>
                    setFilters((current) => ({
                      ...current,
                      includeMode: current.includeMode === "all" ? "any" : "all",
                    }))
                  }
                  style={chipButtonStyle}
                >
                  match {filters.includeMode}
                </button>
              )}
              <label style={{ display: "flex", alignItems: "center", gap: "var(--space-1)" }}>
                <input
                  type="checkbox"
                  checked={filters.untaggedOnly}
                  onChange={(event) =>
                    setFilters((current) => ({ ...current, untaggedOnly: event.target.checked }))
                  }
                />
                Untagged only
              </label>
            </>
          )}

          {filtersActive && (
            <button type="button" onClick={() => setFilters(EMPTY_BOARD_FILTERS)} style={chipButtonStyle}>
              Clear filters
            </button>
          )}

          <span style={{ flex: 1 }} />
          Read-only view · specs move through these states via the `lmbrain-mcp` tools
        </div>

        {(filters.includeTags.length > 0 || filters.excludeTags.length > 0) && (
          <div
            aria-label="Active tag filters"
            style={{ display: "flex", flexWrap: "wrap", gap: "var(--space-1)", marginTop: "var(--space-2)" }}
          >
            {filters.includeTags.map((tag) => (
              <button
                key={`include-${tag}`}
                type="button"
                aria-label={`Remove include filter ${tag}`}
                onClick={() =>
                  setFilters((current) => ({
                    ...current,
                    includeTags: toggleValue(current.includeTags, tag),
                  }))
                }
                style={{ ...chipButtonStyle, color: "#bcaef6" }}
              >
                {tag} ×
              </button>
            ))}
            {filters.excludeTags.map((tag) => (
              <button
                key={`exclude-${tag}`}
                type="button"
                aria-label={`Remove exclude filter ${tag}`}
                onClick={() =>
                  setFilters((current) => ({
                    ...current,
                    excludeTags: toggleValue(current.excludeTags, tag),
                  }))
                }
                style={{ ...chipButtonStyle, color: "#e0a23a" }}
              >
                −{tag} ×
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Columns */}
      <div style={{ flex: 1, minHeight: 0, overflowX: "auto", overflowY: "hidden", padding: "var(--space-4) var(--page-gutter)" }}>
        <div style={{ display: "flex", gap: 14, height: "100%", minWidth: "max-content" }}>
          {COLUMNS.map((col) => {
            const specs = specsByStatus(col.status);
            return (
              <div
                key={col.status}
                style={{
                  width: 262,
                  flex: "none",
                  display: "flex",
                  flexDirection: "column",
                  minHeight: 0,
                }}
              >
                <div
                  style={{ display: "flex", alignItems: "center", gap: 8, padding: "0 4px 11px" }}
                >
                  <span
                    style={{ width: 9, height: 9, borderRadius: "50%", background: col.color }}
                  />
                  <span style={{ fontSize: "var(--text-sm)", fontWeight: 700, color: "var(--text-primary)" }}>
                    {col.label}
                  </span>
                  <span
                    title={filtersActive ? "shown / total in this status" : undefined}
                    style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "var(--text-muted)" }}
                  >
                    {filtersActive ? `${specs.length}/${totalByStatus(col.status)}` : specs.length}
                  </span>
                </div>
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 9,
                    flex: 1,
                    minHeight: 0,
                    overflowY: "auto",
                    paddingRight: 2,
                  }}
                >
                  {specs.map((spec) => (
                    <SpecCard
                      key={spec.id}
                      spec={spec}
                      activeDebtCount={(state.debts ?? []).filter((debt) =>
                        ["open", "planned", "deferred"].includes(debt.status)
                        && (debt.origin_artifact === spec.id
                          || debt.related_specs.includes(spec.id)
                          || debt.target_specs.includes(spec.id))
                      ).length}
                      dependencyBlockers={(spec.depends_on ?? []).filter((id) =>
                        state.specs.find((candidate) => candidate.id === id)?.status !== "done"
                      )}
                      onClick={() => openSpec(spec)}
                    />
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function SpecCard({
  spec,
  activeDebtCount,
  dependencyBlockers,
  onClick,
}: {
  spec: Spec;
  activeDebtCount: number;
  dependencyBlockers: string[];
  onClick: () => void;
}) {
  const { done, total } = criteriaProgress(spec.body);
  const isMalformed = !!spec.malformed;

  return (
    <div
      onClick={onClick}
      style={{
        background: "var(--bg-tertiary)",
        border: isMalformed ? "1px solid #e0584a" : "1px solid #262330",
        borderRadius: 11,
        padding: "12px 13px",
        cursor: "pointer",
        display: "flex",
        flexDirection: "column",
        gap: 8,
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "#3a3446";
        e.currentTarget.style.background = "#181520";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "#262330";
        e.currentTarget.style.background = "var(--bg-tertiary)";
      }}
    >
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span
          style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}
        >
          {spec.id}
        </span>
        {isMalformed && (
          <span
            style={{
              fontSize: "var(--text-2xs)",
              fontWeight: 700,
              color: "#e0584a",
              background: "rgba(224,88,74,0.13)",
              borderRadius: 5,
              padding: "2px 7px",
              letterSpacing: "0.03em",
            }}
          >
            MALFORMED
          </span>
        )}
      </div>
      <div style={{ fontSize: "var(--text-md)", fontWeight: 600, lineHeight: 1.35, color: "var(--text-primary)" }}>
        {spec.title}
      </div>
      {(spec.tags ?? []).length > 0 && (
        <div style={{ display: "flex", flexWrap: "wrap", gap: "var(--space-1)" }}>
          {/* Three chips plus an overflow count: enough to explain why a spec
              matched a filter without turning the card into a tag cloud. */}
          {(spec.tags ?? []).slice(0, 3).map((tag) => (
            <span
              key={tag}
              style={{
                fontSize: "var(--text-2xs)",
                color: "var(--text-secondary)",
                background: "#1a1722",
                border: "1px solid var(--border-secondary)",
                borderRadius: "var(--radius-pill)",
                padding: "1px 7px",
              }}
            >
              {tag}
            </span>
          ))}
          {(spec.tags ?? []).length > 3 && (
            <span
              title={(spec.tags ?? []).join(", ")}
              style={{ fontSize: "var(--text-2xs)", color: "var(--text-muted)" }}
            >
              +{(spec.tags ?? []).length - 3}
            </span>
          )}
        </div>
      )}
      {spec.status === "backlog" && (spec.parking_events?.length ?? 0) > 0 && (
        <div
          title={spec.parking_events?.at(-1)?.reason}
          style={{ fontSize: "var(--text-xs)", color: "#bcaef6" }}
        >
          Parked · readiness expired
        </div>
      )}
      <div style={{ display: "flex", alignItems: "center", gap: 8, marginTop: 1 }}>
        {spec.capability_tier && (
          <span
            aria-label={`Capability tier ${spec.capability_tier}${spec.thinking_level ? `, ${spec.thinking_level} reasoning` : ""}`}
            title={`Implementation estimate: ${spec.capability_tier}${spec.thinking_level ? ` · ${spec.thinking_level} reasoning` : ""}`}
            style={{
              fontSize: "var(--text-2xs)",
              fontWeight: 700,
              textTransform: "uppercase",
              letterSpacing: ".05em",
              color: TIER_COLORS[spec.capability_tier] ?? "var(--text-secondary)",
              border: `1px solid ${TIER_COLORS[spec.capability_tier] ?? "var(--border-secondary)"}`,
              borderRadius: "var(--radius-sm)",
              padding: "1px 6px",
            }}
          >
            {spec.capability_tier}
          </span>
        )}
        {spec.recommended_agent && (
          <span
            style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-xs)", color: "#bcaef6" }}
          >
            {spec.recommended_agent}
          </span>
        )}
        {total > 0 && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-xs)",
              color: done === total ? "var(--green)" : "var(--text-secondary)",
              background: "#1a1722",
              borderRadius: 5,
              padding: "2px 6px",
            }}
          >
            {done}/{total}
          </span>
        )}
        {activeDebtCount > 0 && (
          <span
            aria-label={`${activeDebtCount} active debts`}
            title={`${activeDebtCount} active debt${activeDebtCount > 1 ? "s" : ""} linked to this spec`}
            style={{
              fontSize: "var(--text-xs)",
              color: "#d9b86d",
              display: "inline-flex",
              alignItems: "center",
              gap: 3,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
              link
            </i>{" "}
            {activeDebtCount}
          </span>
        )}
        {dependencyBlockers.length > 0 && (
          <span
            aria-label={`Blocked by hard dependencies: ${dependencyBlockers.join(", ")}`}
            title={`Ready after ${dependencyBlockers.join(", ")}`}
            style={{ fontSize: "var(--text-xs)", color: "#e0a23a" }}
          >
            ⛓ {dependencyBlockers.length}
          </span>
        )}
        <span style={{ flex: 1 }} />
        <span style={{ fontSize: "var(--text-xs)", color: "var(--text-muted)", whiteSpace: "nowrap" }}>
          {spec.updated}
        </span>
      </div>
    </div>
  );
}
