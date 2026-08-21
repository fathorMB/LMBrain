import { useMemo, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import {
  EMPTY_BOARD_FILTERS,
  collectTagVocabulary,
  hasActiveBoardFilters,
  matchesBoardFilters,
  type BoardFilters,
} from "../../lib/boardFilters";
import type { SpecStatus } from "../../types";
import { SpecCard } from "./SpecCard";
import { TaskboardFilters } from "./TaskboardFilters";

const COLUMNS: { status: SpecStatus; label: string; color: string }[] = [
  { status: "backlog", label: "Backlog", color: "var(--text-tertiary)" },
  { status: "ready", label: "Ready", color: "#8a8d99" },
  { status: "working", label: "Working", color: "#5b8def" },
  { status: "review", label: "Review", color: "#e0a23a" },
  { status: "done", label: "Done", color: "#46b07d" },
  { status: "discarded", label: "Discarded", color: "#e0584a" },
];

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
      {/* Header & Filter Toolbar */}
      <TaskboardFilters
        filters={filters}
        setFilters={setFilters}
        tagVocabulary={tagVocabulary}
      />

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
