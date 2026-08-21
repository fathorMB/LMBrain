import { useMemo, useState, type CSSProperties } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import {
  buildInboundIndex,
  collectAttentionItems,
  collectDecisionTags,
  EMPTY_DECISION_FILTERS,
  groupDecisions,
  hasActiveDecisionFilters,
  indexById,
  matchesDecisionFilters,
  type DecisionFilters,
  type DecisionSort,
} from "../../lib/decisionIndex";
import type { AdrStatus } from "../../types";
import { CardGrid, EmptyState, PageHeader, PageShell, Toolbar } from "../Shared/PageLayout";
import { AttentionBand } from "./AttentionBand";
import { DecisionCard } from "./DecisionCard";

const selectStyle: CSSProperties = {
  background: "var(--bg-tertiary)",
  color: "var(--text-primary)",
  border: "1px solid var(--border-primary)",
  borderRadius: "var(--radius-sm)",
  padding: "5px 7px",
};

const STATUSES: AdrStatus[] = ["accepted", "proposed", "superseded", "deprecated", "rejected"];

export function DecisionsList() {
  const { state, openDetailArtifact } = useWorkspace();
  const [filters, setFilters] = useState<DecisionFilters>(EMPTY_DECISION_FILTERS);
  const [historyOpen, setHistoryOpen] = useState(false);

  const adrs = state.adrs;
  const byId = useMemo(() => indexById(adrs), [adrs]);
  const inbound = useMemo(
    () => buildInboundIndex(state.specs, state.debts),
    [state.specs, state.debts],
  );
  const attention = useMemo(() => collectAttentionItems(adrs), [adrs]);
  const tags = useMemo(() => collectDecisionTags(adrs), [adrs]);

  const filtersActive = hasActiveDecisionFilters(filters);
  const visible = useMemo(
    () => adrs.filter((adr) => matchesDecisionFilters(adr, filters)),
    [adrs, filters],
  );
  const groups = useMemo(() => groupDecisions(visible, filters.sort), [visible, filters.sort]);

  const open = (adr: { title: string; path: string }) =>
    openDetailArtifact({ title: adr.title, path: adr.path });

  return (
    <PageShell archetype="dense">
      <PageHeader
        title="Decisions"
        description={
          <>
            Architecture decision records in{" "}
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-sm)",
                color: "var(--text-secondary)",
              }}
            >
              .lmbrain/decisions/
            </span>
            .
          </>
        }
      />

      {adrs.length === 0 ? (
        <EmptyState>No decisions recorded yet.</EmptyState>
      ) : (
        <>
          {attention.length > 0 && <AttentionBand items={attention} onOpen={open} byId={byId} />}

          <Toolbar>
            <input
              type="search"
              aria-label="Search decisions"
              placeholder="Search by ID or title"
              value={filters.query}
              onChange={(event) => setFilters({ ...filters, query: event.target.value })}
              style={{ ...selectStyle, minWidth: 200 }}
            />
            <select
              className="app-select"
              aria-label="Status"
              value={filters.status}
              onChange={(event) =>
                setFilters({ ...filters, status: event.target.value as AdrStatus | "" })
              }
              style={selectStyle}
            >
              <option value="">All statuses</option>
              {STATUSES.map((status) => (
                <option key={status} value={status}>
                  {status}
                </option>
              ))}
            </select>
            {tags.length > 0 && (
              <select
                className="app-select"
                aria-label="Tag"
                value={filters.tag}
                onChange={(event) => setFilters({ ...filters, tag: event.target.value })}
                style={selectStyle}
              >
                <option value="">All tags</option>
                {tags.map((tag) => (
                  <option key={tag} value={tag}>
                    {tag}
                  </option>
                ))}
              </select>
            )}
            <select
              className="app-select"
              aria-label="Sort"
              value={filters.sort}
              onChange={(event) =>
                setFilters({ ...filters, sort: event.target.value as DecisionSort })
              }
              style={selectStyle}
            >
              <option value="recent">Most recent</option>
              <option value="id">By ID</option>
            </select>
            {filtersActive && (
              <button
                type="button"
                onClick={() => setFilters({ ...EMPTY_DECISION_FILTERS, sort: filters.sort })}
                style={{ ...selectStyle, cursor: "pointer" }}
              >
                Clear filters
              </button>
            )}
          </Toolbar>

          {groups.length === 0 ? (
            <EmptyState>No decisions match these filters.</EmptyState>
          ) : (
            groups.map((group) => {
              const collapsible = group.key === "historical";
              const expanded = collapsible ? historyOpen : true;
              const headingId = `decision-group-${group.key}`;
              return (
                <section
                  key={group.key}
                  aria-labelledby={headingId}
                  style={{ marginBottom: "var(--space-5)" }}
                >
                  {collapsible ? (
                    <button
                      type="button"
                      id={headingId}
                      aria-expanded={expanded}
                      onClick={() => setHistoryOpen((value) => !value)}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: "var(--space-2)",
                        background: "none",
                        border: "none",
                        padding: 0,
                        marginBottom: "var(--space-3)",
                        color: "var(--text-secondary)",
                        font: "inherit",
                        fontSize: "var(--text-sm)",
                        fontWeight: 700,
                        cursor: "pointer",
                      }}
                    >
                      <i className="material-symbols-outlined" style={{ fontSize: "var(--text-md)" }}>
                        {expanded ? "expand_more" : "chevron_right"}
                      </i>
                      {group.label} · {group.decisions.length}
                    </button>
                  ) : (
                    <h2
                      id={headingId}
                      style={{
                        fontSize: "var(--text-sm)",
                        fontWeight: 700,
                        color: "var(--text-secondary)",
                        margin: "0 0 var(--space-3)",
                      }}
                    >
                      {group.label} · {group.decisions.length}
                    </h2>
                  )}
                  {expanded && (
                    <CardGrid>
                      {group.decisions.map((adr) => (
                        <DecisionCard
                          key={adr.id}
                          adr={adr}
                          byId={byId}
                          inbound={inbound.get(adr.id.toUpperCase())?.length ?? 0}
                          onOpen={open}
                        />
                      ))}
                    </CardGrid>
                  )}
                </section>
              );
            })
          )}
        </>
      )}
    </PageShell>
  );
}
