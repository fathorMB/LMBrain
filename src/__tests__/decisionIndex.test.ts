import { describe, expect, it } from "vitest";
import {
  EMPTY_DECISION_FILTERS,
  buildInboundIndex,
  collectAttentionItems,
  collectDecisionTags,
  groupDecisions,
  groupKeyFor,
  hasActiveDecisionFilters,
  indexById,
  matchesDecisionFilters,
  supersessionChain,
} from "../lib/decisionIndex";
import type { Adr, Finding, Spec } from "../types";

function adr(overrides: Partial<Adr> = {}): Adr {
  return {
    id: "ADR-001",
    title: "A decision",
    status: "accepted",
    decision_date: "2026-07-01",
    decider: "Project Lead",
    body: "",
    path: ".lmbrain/decisions/ADR-001.md",
    created: "2026-07-01",
    updated: "2026-07-01",
    tags: [],
    links: [],
    supersedes: [],
    superseded_by: [],
    ...overrides,
  };
}

describe("grouping by authority", () => {
  it("maps every status onto one of three groups", () => {
    expect(groupKeyFor("accepted")).toBe("authoritative");
    expect(groupKeyFor("proposed")).toBe("awaiting");
    expect(groupKeyFor("superseded")).toBe("historical");
    expect(groupKeyFor("deprecated")).toBe("historical");
    expect(groupKeyFor("rejected")).toBe("historical");
  });

  it("treats an unrecognized status as awaiting rather than dropping it", () => {
    expect(groupKeyFor("invented")).toBe("awaiting");
    const groups = groupDecisions([adr({ status: "invented" as Adr["status"] })]);
    expect(groups.map((group) => group.key)).toEqual(["awaiting"]);
  });

  it("omits empty groups instead of rendering a zero", () => {
    const groups = groupDecisions([adr(), adr({ id: "ADR-002" })]);
    expect(groups).toHaveLength(1);
    expect(groups[0].label).toBe("Authoritative");
  });

  it("sorts by recency, falling back to the update date", () => {
    const groups = groupDecisions([
      adr({ id: "ADR-001", decision_date: "2026-01-01" }),
      adr({ id: "ADR-002", decision_date: null, updated: "2026-09-01" }),
      adr({ id: "ADR-003", decision_date: "2026-05-01" }),
    ]);
    expect(groups[0].decisions.map((decision) => decision.id)).toEqual([
      "ADR-002",
      "ADR-003",
      "ADR-001",
    ]);
  });

  it("sorts by ID on request", () => {
    const groups = groupDecisions(
      [adr({ id: "ADR-003" }), adr({ id: "ADR-001" })],
      "id",
    );
    expect(groups[0].decisions.map((decision) => decision.id)).toEqual(["ADR-001", "ADR-003"]);
  });
});

describe("filtering", () => {
  const layout = adr({ id: "ADR-001", title: "Layout system", tags: ["Design"] });
  const branching = adr({ id: "ADR-002", title: "Branching", status: "proposed" });

  it("matches ID and title case-insensitively", () => {
    const filters = { ...EMPTY_DECISION_FILTERS, query: "LAYOUT" };
    expect(matchesDecisionFilters(layout, filters)).toBe(true);
    expect(matchesDecisionFilters(branching, filters)).toBe(false);
    expect(
      matchesDecisionFilters(layout, { ...EMPTY_DECISION_FILTERS, query: "adr-001" }),
    ).toBe(true);
  });

  it("composes status and tag", () => {
    expect(
      matchesDecisionFilters(layout, {
        ...EMPTY_DECISION_FILTERS,
        status: "accepted",
        tag: "design",
      }),
    ).toBe(true);
    expect(
      matchesDecisionFilters(layout, { ...EMPTY_DECISION_FILTERS, status: "proposed" }),
    ).toBe(false);
  });

  it("does not count sort order as an active filter", () => {
    expect(hasActiveDecisionFilters(EMPTY_DECISION_FILTERS)).toBe(false);
    expect(hasActiveDecisionFilters({ ...EMPTY_DECISION_FILTERS, sort: "id" })).toBe(false);
    expect(hasActiveDecisionFilters({ ...EMPTY_DECISION_FILTERS, query: " " })).toBe(false);
    expect(hasActiveDecisionFilters({ ...EMPTY_DECISION_FILTERS, tag: "design" })).toBe(true);
  });

  it("collects a normalized, sorted tag vocabulary", () => {
    expect(collectDecisionTags([layout, adr({ tags: ["workflow", "design"] })])).toEqual([
      "design",
      "workflow",
    ]);
  });
});

describe("inbound references", () => {
  it("indexes specs and findings that cite a decision, without duplicates", () => {
    const specs = [
      { id: "SPEC-001", title: "One", related_decisions: ["ADR-001", "adr-001"] },
      { id: "SPEC-002", title: "Two", related_decisions: ["ADR-002"] },
    ] as unknown as Spec[];
    const findings = [
      { id: "FINDING-001", title: "F", related_decisions: [" ADR-001 "] },
    ] as unknown as Finding[];

    const index = buildInboundIndex(specs, findings);
    expect(index.get("ADR-001")?.map((entry) => entry.id)).toEqual(["SPEC-001", "FINDING-001"]);
    expect(index.get("ADR-002")).toHaveLength(1);
    expect(index.get("ADR-999")).toBeUndefined();
  });

  it("tolerates artifacts with no related decisions", () => {
    const specs = [{ id: "SPEC-001", title: "One" }] as unknown as Spec[];
    expect(buildInboundIndex(specs, [] as unknown as Finding[]).size).toBe(0);
  });
});

describe("supersession chains", () => {
  it("walks backwards through predecessors", () => {
    const adrs = [
      adr({ id: "ADR-003", supersedes: ["ADR-002"] }),
      adr({ id: "ADR-002", supersedes: ["ADR-001"] }),
      adr({ id: "ADR-001" }),
    ];
    const byId = indexById(adrs);
    expect(supersessionChain(adrs[0], byId, "supersedes").map((a) => a.id)).toEqual([
      "ADR-002",
      "ADR-001",
    ]);
  });

  it("stops at the first repeated ID so a cycle cannot hang the render", () => {
    const adrs = [
      adr({ id: "ADR-001", supersedes: ["ADR-002"] }),
      adr({ id: "ADR-002", supersedes: ["ADR-001"] }),
    ];
    const byId = indexById(adrs);
    expect(supersessionChain(adrs[0], byId, "supersedes").map((a) => a.id)).toEqual(["ADR-002"]);
  });

  it("stops at a reference that does not resolve", () => {
    const adrs = [adr({ id: "ADR-001", supersedes: ["ADR-404"] })];
    expect(supersessionChain(adrs[0], indexById(adrs), "supersedes")).toEqual([]);
  });
});

describe("attention items", () => {
  it("reports a predecessor that never left accepted", () => {
    const items = collectAttentionItems([
      adr({ id: "ADR-010", supersedes: ["ADR-009"] }),
      adr({ id: "ADR-009" }),
    ]);
    expect(items).toHaveLength(1);
    expect(items[0]).toMatchObject({ kind: "integrity", adrId: "ADR-009" });
  });

  it("reports a one-sided relationship even once the predecessor is retired", () => {
    const items = collectAttentionItems([
      adr({ id: "ADR-010", supersedes: ["ADR-009"] }),
      adr({ id: "ADR-009", status: "superseded", superseded_by: [] }),
    ]);
    expect(items[0].message).toContain("does not record ADR-010");
  });

  it("stays silent on a consistent pair", () => {
    const items = collectAttentionItems([
      adr({ id: "ADR-010", supersedes: ["ADR-009"] }),
      adr({ id: "ADR-009", status: "superseded", superseded_by: ["ADR-010"] }),
    ]);
    expect(items).toEqual([]);
  });

  it("treats a proposal's supersession claim as pending, not broken", () => {
    const items = collectAttentionItems([
      adr({ id: "ADR-014", status: "proposed", supersedes: ["ADR-013"] }),
      adr({ id: "ADR-013" }),
    ]);
    expect(items.map((item) => item.kind)).toEqual(["pending"]);
  });

  it("reports a reference to a decision that does not exist", () => {
    const items = collectAttentionItems([adr({ id: "ADR-010", supersedes: ["ADR-404"] })]);
    expect(items[0].message).toContain("does not exist");
  });

  it("orders integrity before malformed before pending", () => {
    const items = collectAttentionItems([
      adr({ id: "ADR-002", status: "proposed" }),
      adr({ id: "ADR-003", malformed: true }),
      adr({ id: "ADR-010", supersedes: ["ADR-009"] }),
      adr({ id: "ADR-009" }),
    ]);
    expect(items.map((item) => item.kind)).toEqual(["integrity", "malformed", "pending"]);
  });
});
