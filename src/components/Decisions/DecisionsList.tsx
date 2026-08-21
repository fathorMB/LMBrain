import { useEffect, useMemo, useState, type CSSProperties } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getAdrs } from "../../lib/commands";
import {
  EMPTY_DECISION_FILTERS,
  buildInboundIndex,
  collectAttentionItems,
  collectDecisionTags,
  groupDecisions,
  hasActiveDecisionFilters,
  indexById,
  matchesDecisionFilters,
  supersessionChain,
  type AttentionItem,
  type DecisionFilters,
  type DecisionSort,
} from "../../lib/decisionIndex";
import type { Adr, AdrStatus } from "../../types";
import { CardGrid, EmptyState, PageHeader, PageShell, Toolbar } from "../Shared/PageLayout";

const STATUS_COLORS: Record<AdrStatus, { color: string; bg: string }> = {
  accepted: { color: "#46b07d", bg: "rgba(70,176,125,.12)" },
  proposed: { color: "#8a8d99", bg: "rgba(139,141,152,.12)" },
  superseded: { color: "#e0a23a", bg: "rgba(224,162,58,.12)" },
  deprecated: { color: "#c07ad8", bg: "rgba(192,122,216,.12)" },
  // `rejected` used to fall through to the `proposed` grey, so a refused
  // decision read as one still awaiting an answer.
  rejected: { color: "#e0584a", bg: "rgba(224,88,74,.12)" },
};

const ATTENTION_ICONS = {
  integrity: { icon: "link_off", color: "#e0a23a" },
  malformed: { icon: "warning", color: "#e0584a" },
  pending: { icon: "pending", color: "var(--text-tertiary)" },
} as const;

const selectStyle = {
  background: "var(--bg-tertiary)",
  color: "var(--text-primary)",
  border: "1px solid var(--border-primary)",
  borderRadius: "var(--radius-sm)",
  padding: "5px 7px",
} as const;

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

function AttentionBand({
  items,
  byId,
  onOpen,
}: {
  items: AttentionItem[];
  byId: Map<string, Adr>;
  onOpen: (adr: { title: string; path: string }) => void;
}) {
  return (
    <section
      aria-label="Needs attention"
      style={{
        border: "1px solid var(--border-secondary)",
        borderRadius: "var(--radius-lg)",
        background: "var(--bg-tertiary)",
        padding: "var(--space-3)",
        marginBottom: "var(--space-4)",
      }}
    >
      <h2
        style={{
          fontSize: "var(--text-sm)",
          fontWeight: 700,
          color: "var(--text-secondary)",
          margin: "0 0 var(--space-2)",
        }}
      >
        Needs attention · {items.length}
      </h2>
      <ul style={{ listStyle: "none", margin: 0, padding: 0, display: "grid", gap: "var(--space-1)" }}>
        {items.map((item) => {
          const visual = ATTENTION_ICONS[item.kind];
          const target = byId.get(item.adrId.toUpperCase());
          return (
            <li key={`${item.kind}:${item.adrId}:${item.message}`}>
              <button
                type="button"
                onClick={() => onOpen({ title: target?.title ?? item.adrId, path: item.path })}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "var(--space-2)",
                  width: "100%",
                  textAlign: "left",
                  background: "none",
                  border: "none",
                  padding: "var(--space-1) 0",
                  font: "inherit",
                  fontSize: "var(--text-sm)",
                  color: "var(--text-secondary)",
                  cursor: "pointer",
                }}
              >
                <i
                  className="material-symbols-outlined"
                  aria-hidden="true"
                  style={{ fontSize: "var(--text-md)", color: visual.color }}
                >
                  {visual.icon}
                </i>
                {item.message}
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}

function DecisionCard({
  adr,
  byId,
  inbound,
  onOpen,
}: {
  adr: Adr;
  byId: Map<string, Adr>;
  inbound: number;
  onOpen: (adr: { title: string; path: string }) => void;
}) {
  const status = STATUS_COLORS[adr.status] ?? STATUS_COLORS.proposed;
  const isMalformed = Boolean(adr.malformed);
  const retires = supersessionChain(adr, byId, "supersedes")[0];
  const retiredBy = supersessionChain(adr, byId, "superseded_by")[0];
  const provenance = [adr.decision_date, adr.decider].filter(Boolean).join(" · ");

  const cardStyle: CSSProperties = {
    display: "flex",
    flexDirection: "column",
    gap: "var(--space-2)",
    width: "100%",
    textAlign: "left",
    font: "inherit",
    color: "inherit",
    background: "var(--bg-tertiary)",
    border: `1px solid ${isMalformed ? "#e0584a" : "var(--border-secondary)"}`,
    borderRadius: "var(--radius-lg)",
    padding: "var(--space-3)",
  };

  return (
    <button
      type="button"
      aria-label={`${adr.id}, ${adr.title}, ${adr.status}`}
      onClick={() => onOpen(adr)}
      style={cardStyle}
      onMouseEnter={(event) => {
        event.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "var(--border-hover)";
      }}
      onMouseLeave={(event) => {
        event.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "var(--border-secondary)";
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-2)" }}>
        <span
          style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)", color: "#bcaef6" }}
        >
          {adr.id}
        </span>
        <span style={{ flex: 1 }} />
        {isMalformed && (
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              gap: "var(--space-1)",
              fontSize: "var(--text-2xs)",
              fontWeight: 700,
              color: "#e0584a",
              background: "rgba(224,88,74,0.13)",
              borderRadius: "var(--radius-sm)",
              padding: "var(--space-0) var(--space-2)",
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: "var(--text-xs)" }}>
              warning
            </i>
            MALFORMED
          </span>
        )}
        <span
          style={{
            fontSize: "var(--text-2xs)",
            fontWeight: 700,
            color: status.color,
            background: status.bg,
            borderRadius: "var(--radius-sm)",
            padding: "var(--space-0) var(--space-2)",
          }}
        >
          {adr.status.toUpperCase()}
        </span>
      </div>

      <div style={{ fontSize: "var(--text-lg)", fontWeight: 600, color: "var(--text-primary)" }}>
        {adr.title}
      </div>

      {provenance && (
        <div style={{ fontSize: "var(--text-xs)", color: "var(--text-tertiary)" }}>{provenance}</div>
      )}

      {(retires || retiredBy || inbound > 0) && (
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: "var(--space-3)",
            fontSize: "var(--text-xs)",
            color: "var(--text-tertiary)",
          }}
        >
          {retires && <span>replaces {retires.id}</span>}
          {retiredBy && <span>replaced by {retiredBy.id}</span>}
          {inbound > 0 && (
            <span>
              {inbound} {inbound === 1 ? "reference" : "references"}
            </span>
          )}
        </div>
      )}
    </button>
  );
}
