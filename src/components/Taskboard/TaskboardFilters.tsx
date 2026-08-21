import {
  CAPABILITY_TIERS,
  EMPTY_BOARD_FILTERS,
  hasActiveBoardFilters,
  toggleValue,
  type BoardFilters,
} from "../../lib/boardFilters";

export interface TaskboardFiltersProps {
  filters: BoardFilters;
  setFilters: React.Dispatch<React.SetStateAction<BoardFilters>>;
  tagVocabulary: string[];
}

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

export function TaskboardFilters({
  filters,
  setFilters,
  tagVocabulary,
}: TaskboardFiltersProps) {
  const filtersActive = hasActiveBoardFilters(filters);

  return (
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
  );
}
