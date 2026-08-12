import type { Adr, AdrStatus, Debt, Spec } from "../types";

/**
 * Pure model behind the Decisions view (issue #48). Everything here is derived
 * from workspace state that is already loaded, so the page needs no new command
 * and no extra I/O.
 */

export type DecisionGroupKey = "authoritative" | "awaiting" | "historical";

export interface DecisionGroup {
  key: DecisionGroupKey;
  label: string;
  decisions: Adr[];
}

/**
 * Three groups rather than five status buckets: the operator's first question
 * is "what holds right now", and single-status groups would hold one or two
 * records at realistic collection sizes.
 */
const GROUP_FOR_STATUS: Record<AdrStatus, DecisionGroupKey> = {
  accepted: "authoritative",
  proposed: "awaiting",
  superseded: "historical",
  deprecated: "historical",
  rejected: "historical",
};

const GROUP_ORDER: { key: DecisionGroupKey; label: string }[] = [
  { key: "authoritative", label: "Authoritative" },
  { key: "awaiting", label: "Awaiting decision" },
  { key: "historical", label: "Historical" },
];

export function groupKeyFor(status: string): DecisionGroupKey {
  return GROUP_FOR_STATUS[status as AdrStatus] ?? "awaiting";
}

function ids(values: string[] | null | undefined): string[] {
  return (values ?? [])
    .map((value) => String(value).trim().toUpperCase())
    .filter((value) => value.length > 0);
}

/** Most recent first, on the decision date, falling back to the update date. */
export function compareByRecency(left: Adr, right: Adr): number {
  const key = (adr: Adr) => adr.decision_date || adr.updated || adr.created || "";
  return key(right).localeCompare(key(left));
}

export function compareById(left: Adr, right: Adr): number {
  return left.id.localeCompare(right.id);
}

export type DecisionSort = "recent" | "id";

export interface DecisionFilters {
  query: string;
  status: AdrStatus | "";
  tag: string;
  sort: DecisionSort;
}

export const EMPTY_DECISION_FILTERS: DecisionFilters = {
  query: "",
  status: "",
  tag: "",
  sort: "recent",
};

export function hasActiveDecisionFilters(filters: DecisionFilters): boolean {
  return filters.query.trim() !== "" || filters.status !== "" || filters.tag !== "";
}

export function matchesDecisionFilters(adr: Adr, filters: DecisionFilters): boolean {
  const query = filters.query.trim().toLowerCase();
  if (query) {
    const haystack = `${adr.id} ${adr.title}`.toLowerCase();
    if (!haystack.includes(query)) return false;
  }
  if (filters.status && adr.status !== filters.status) return false;
  if (filters.tag) {
    const wanted = filters.tag.trim().toLowerCase();
    const tags = (adr.tags ?? []).map((tag) => tag.trim().toLowerCase());
    if (!tags.includes(wanted)) return false;
  }
  return true;
}

export function collectDecisionTags(adrs: Adr[]): string[] {
  const seen = new Set<string>();
  for (const adr of adrs) {
    for (const tag of adr.tags ?? []) {
      const normalized = tag.trim().toLowerCase();
      if (normalized) seen.add(normalized);
    }
  }
  return [...seen].sort();
}

/** Empty groups are omitted entirely rather than rendered with a zero. */
export function groupDecisions(adrs: Adr[], sort: DecisionSort = "recent"): DecisionGroup[] {
  const compare = sort === "id" ? compareById : compareByRecency;
  return GROUP_ORDER.map(({ key, label }) => ({
    key,
    label,
    decisions: adrs.filter((adr) => groupKeyFor(adr.status) === key).sort(compare),
  })).filter((group) => group.decisions.length > 0);
}

export interface InboundReference {
  id: string;
  title: string;
  kind: "spec" | "debt";
}

/**
 * Reverse index: which specs and debts cite each ADR. The forward edges live
 * in `related_decisions` on the citing artifact, so this is the only place the
 * ADR side of the relationship can be assembled.
 */
export function buildInboundIndex(
  specs: Spec[],
  debts: Debt[],
): Map<string, InboundReference[]> {
  const index = new Map<string, InboundReference[]>();
  const add = (adrId: string, reference: InboundReference) => {
    const existing = index.get(adrId);
    if (existing) {
      if (!existing.some((entry) => entry.id === reference.id)) existing.push(reference);
    } else {
      index.set(adrId, [reference]);
    }
  };
  for (const spec of specs ?? []) {
    for (const adrId of ids(spec.related_decisions)) {
      add(adrId, { id: spec.id, title: spec.title, kind: "spec" });
    }
  }
  for (const debt of debts ?? []) {
    for (const adrId of ids(debt.related_decisions)) {
      add(adrId, { id: debt.id, title: debt.title, kind: "debt" });
    }
  }
  return index;
}

/**
 * Walk a supersession chain in one direction, stopping at the first repeated
 * ID. A malformed pair of records can point at each other; recursing would
 * hang the render.
 */
export function supersessionChain(
  start: Adr,
  byId: Map<string, Adr>,
  direction: "supersedes" | "superseded_by",
): Adr[] {
  const chain: Adr[] = [];
  const seen = new Set<string>([start.id.toUpperCase()]);
  let current: Adr | undefined = start;
  while (current) {
    const nextId: string | undefined = ids(current[direction])[0];
    if (!nextId || seen.has(nextId)) break;
    seen.add(nextId);
    const next = byId.get(nextId);
    if (!next) break;
    chain.push(next);
    current = next;
  }
  return chain;
}

export function indexById(adrs: Adr[]): Map<string, Adr> {
  return new Map(adrs.map((adr) => [adr.id.toUpperCase(), adr]));
}

export type AttentionKind = "integrity" | "malformed" | "pending";

export interface AttentionItem {
  kind: AttentionKind;
  adrId: string;
  path: string;
  message: string;
}

const ATTENTION_ORDER: Record<AttentionKind, number> = {
  integrity: 0,
  malformed: 1,
  pending: 2,
};

/**
 * What the operator has to act on, in severity order. Mirrors the
 * `diagnose_decisions` checks in the core so the band and the diagnostics
 * report the same drift, and stays empty when there is nothing to do — an
 * always-present band trains the operator to ignore it.
 */
export function collectAttentionItems(adrs: Adr[]): AttentionItem[] {
  const byId = indexById(adrs);
  const items: AttentionItem[] = [];

  for (const adr of adrs) {
    if (adr.malformed) {
      items.push({
        kind: "malformed",
        adrId: adr.id,
        path: adr.path,
        message: `${adr.id} could not be fully parsed`,
      });
    }

    for (const declared of ids(adr.supersedes)) {
      const target = byId.get(declared);
      if (!target) {
        items.push({
          kind: "integrity",
          adrId: adr.id,
          path: adr.path,
          message: `${adr.id} supersedes ${declared}, which does not exist`,
        });
        continue;
      }
      // A proposal declaring a supersession is a legitimate pending claim:
      // supersession only takes effect once the successor is accepted.
      if (adr.status !== "accepted") continue;
      if (target.status !== "superseded") {
        items.push({
          kind: "integrity",
          adrId: target.id,
          path: target.path,
          message: `${target.id} is still ${target.status} although ${adr.id} supersedes it`,
        });
      } else if (!ids(target.superseded_by).includes(adr.id.toUpperCase())) {
        items.push({
          kind: "integrity",
          adrId: target.id,
          path: target.path,
          message: `${target.id} does not record ${adr.id} in superseded_by`,
        });
      }
    }

    if (adr.status === "proposed") {
      items.push({
        kind: "pending",
        adrId: adr.id,
        path: adr.path,
        message: `${adr.id} awaits an accept or reject decision`,
      });
    }
  }

  return items.sort(
    (left, right) =>
      ATTENTION_ORDER[left.kind] - ATTENTION_ORDER[right.kind] ||
      left.adrId.localeCompare(right.adrId),
  );
}
