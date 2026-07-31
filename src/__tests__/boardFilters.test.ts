import { describe, expect, it } from "vitest";
import {
  EMPTY_BOARD_FILTERS,
  collectTagVocabulary,
  hasActiveBoardFilters,
  isBlockedByDependencies,
  matchesBoardFilters,
  toggleValue,
  type BoardFilters,
} from "../lib/boardFilters";
import type { Spec } from "../types";

function spec(overrides: Partial<Spec> = {}): Spec {
  return {
    id: "SPEC-001",
    title: "A spec",
    status: "ready",
    priority: null,
    area: null,
    milestone: null,
    recommended_agent: null,
    capability_tier: null,
    thinking_level: null,
    depends_on: [],
    skills: [],
    body: "",
    path: ".lmbrain/specs/ready/SPEC-001.md",
    created: "2026-07-01",
    updated: "2026-07-01",
    tags: [],
    links: [],
    related_tasks: [],
    related_decisions: [],
    ...overrides,
  };
}

function filters(overrides: Partial<BoardFilters> = {}): BoardFilters {
  return { ...EMPTY_BOARD_FILTERS, ...overrides };
}

describe("tag filtering", () => {
  const wiki = spec({ id: "SPEC-001", tags: ["wiki", "ux"] });
  const mcp = spec({ id: "SPEC-002", tags: ["mcp"] });
  const untagged = spec({ id: "SPEC-003", tags: [] });
  const all = [wiki, mcp, untagged];

  it("includes any-of by default", () => {
    const active = filters({ includeTags: ["wiki", "mcp"] });
    expect(all.filter((s) => matchesBoardFilters(s, active, all)).map((s) => s.id)).toEqual([
      "SPEC-001",
      "SPEC-002",
    ]);
  });

  it("supports all-of matching", () => {
    const active = filters({ includeTags: ["wiki", "ux"], includeMode: "all" });
    expect(matchesBoardFilters(wiki, active, all)).toBe(true);
    expect(matchesBoardFilters(mcp, active, all)).toBe(false);
  });

  it("excludes regardless of includes", () => {
    const active = filters({ includeTags: ["wiki"], excludeTags: ["ux"] });
    expect(matchesBoardFilters(wiki, active, all)).toBe(false);
  });

  it("matches tags case- and whitespace-insensitively", () => {
    const active = filters({ includeTags: [" WIKI "] });
    expect(matchesBoardFilters(spec({ tags: ["Wiki"] }), active, all)).toBe(true);
  });

  it("surfaces untagged specs through a dedicated toggle", () => {
    const active = filters({ untaggedOnly: true });
    expect(all.filter((s) => matchesBoardFilters(s, active, all)).map((s) => s.id)).toEqual([
      "SPEC-003",
    ]);
  });

  it("collects a sorted, normalized vocabulary", () => {
    expect(collectTagVocabulary([spec({ tags: ["UX", "wiki"] }), spec({ tags: ["mcp", "ux"] })])).toEqual([
      "mcp",
      "ux",
      "wiki",
    ]);
  });
});

describe("tier filtering", () => {
  const luna = spec({ id: "SPEC-001", capability_tier: "luna" });
  const sol = spec({ id: "SPEC-002", capability_tier: "sol" });
  const legacy = spec({ id: "SPEC-003", capability_tier: null });
  const all = [luna, sol, legacy];

  it("keeps only the selected tier and never matches legacy specs by accident", () => {
    const active = filters({ tiers: ["luna"] });
    expect(all.filter((s) => matchesBoardFilters(s, active, all)).map((s) => s.id)).toEqual([
      "SPEC-001",
    ]);
  });
});

describe("dependency composition", () => {
  const prerequisite = spec({ id: "SPEC-100", status: "working" });
  const blocked = spec({ id: "SPEC-101", depends_on: ["SPEC-100"], tags: ["mcp"] });
  const donePrerequisite = spec({ id: "SPEC-102", status: "done" });
  const unblocked = spec({ id: "SPEC-103", depends_on: ["SPEC-102"], tags: ["wiki"] });
  const all = [prerequisite, blocked, donePrerequisite, unblocked];

  it("detects hard blockers", () => {
    expect(isBlockedByDependencies(blocked, all)).toBe(true);
    expect(isBlockedByDependencies(unblocked, all)).toBe(false);
  });

  it("ANDs the dependency axis with the tag axis", () => {
    const active = filters({ dependency: "blocked", includeTags: ["mcp"] });
    expect(all.filter((s) => matchesBoardFilters(s, active, all)).map((s) => s.id)).toEqual([
      "SPEC-101",
    ]);

    const contradictory = filters({ dependency: "blocked", includeTags: ["wiki"] });
    expect(all.filter((s) => matchesBoardFilters(s, contradictory, all))).toHaveLength(0);
  });

  it("keeps the prerequisites-complete semantics", () => {
    const active = filters({ dependency: "ready-after" });
    expect(all.filter((s) => matchesBoardFilters(s, active, all)).map((s) => s.id)).toEqual([
      "SPEC-103",
    ]);
  });
});

describe("filter state", () => {
  it("reports whether anything is filtering", () => {
    expect(hasActiveBoardFilters(EMPTY_BOARD_FILTERS)).toBe(false);
    expect(hasActiveBoardFilters(filters({ untaggedOnly: true }))).toBe(true);
    expect(hasActiveBoardFilters(filters({ dependency: "blocked" }))).toBe(true);
    expect(hasActiveBoardFilters(filters({ tiers: ["sol"] }))).toBe(true);
  });

  it("toggles values idempotently and ignores empty input", () => {
    expect(toggleValue([], "Wiki")).toEqual(["wiki"]);
    expect(toggleValue(["wiki"], "wiki")).toEqual([]);
    expect(toggleValue(["wiki"], "   ")).toEqual(["wiki"]);
  });

  it("passes everything through when no filter is set", () => {
    const specs = [spec({ tags: [] }), spec({ id: "SPEC-002", tags: ["x1"] })];
    expect(specs.every((s) => matchesBoardFilters(s, EMPTY_BOARD_FILTERS, specs))).toBe(true);
  });

  it("tolerates specs with missing tag and dependency arrays", () => {
    const malformed = { ...spec(), tags: undefined, depends_on: undefined } as unknown as Spec;
    expect(matchesBoardFilters(malformed, EMPTY_BOARD_FILTERS, [malformed])).toBe(true);
    expect(matchesBoardFilters(malformed, filters({ untaggedOnly: true }), [malformed])).toBe(true);
    expect(collectTagVocabulary([malformed])).toEqual([]);
  });
});
