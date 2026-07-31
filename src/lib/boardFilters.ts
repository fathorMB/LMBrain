import type { Spec } from "../types";

/**
 * One filter model for the Board (issue #49). Tags, tier, and the pre-existing
 * dependency view compose as a single predicate instead of accumulating
 * unrelated controls: axes combine with AND, values inside an axis combine as
 * the axis declares.
 */

export type TagMatchMode = "any" | "all";
export type DependencyFilter = "all" | "blocked" | "ready-after";

export interface BoardFilters {
  includeTags: string[];
  includeMode: TagMatchMode;
  excludeTags: string[];
  /** "No tags at all" is not expressible as a tag, so it gets its own toggle. */
  untaggedOnly: boolean;
  tiers: string[];
  dependency: DependencyFilter;
}

export const EMPTY_BOARD_FILTERS: BoardFilters = {
  includeTags: [],
  includeMode: "any",
  excludeTags: [],
  untaggedOnly: false,
  tiers: [],
  dependency: "all",
};

export const CAPABILITY_TIERS = ["luna", "terra", "sol"] as const;

function normalize(value: string | null | undefined): string {
  return (value ?? "").trim().toLowerCase();
}

function specTags(spec: Spec): string[] {
  return (spec.tags ?? []).map(normalize).filter(Boolean);
}

/** A spec is blocked when any hard prerequisite is not done. */
export function isBlockedByDependencies(spec: Spec, specs: Spec[]): boolean {
  return (spec.depends_on ?? []).some(
    (id) => specs.find((candidate) => candidate.id === id)?.status !== "done",
  );
}

export function matchesBoardFilters(spec: Spec, filters: BoardFilters, specs: Spec[]): boolean {
  const tags = specTags(spec);

  if (filters.untaggedOnly && tags.length > 0) return false;

  if (filters.includeTags.length > 0) {
    const wanted = filters.includeTags.map(normalize);
    const matched =
      filters.includeMode === "all"
        ? wanted.every((tag) => tags.includes(tag))
        : wanted.some((tag) => tags.includes(tag));
    if (!matched) return false;
  }

  if (filters.excludeTags.some((tag) => tags.includes(normalize(tag)))) return false;

  if (filters.tiers.length > 0) {
    const tier = normalize(spec.capability_tier);
    if (!filters.tiers.map(normalize).includes(tier)) return false;
  }

  if (filters.dependency !== "all") {
    const dependencies = spec.depends_on ?? [];
    const blocked = isBlockedByDependencies(spec, specs);
    if (filters.dependency === "blocked" && !blocked) return false;
    if (filters.dependency === "ready-after" && !(dependencies.length > 0 && !blocked)) {
      return false;
    }
  }

  return true;
}

export function hasActiveBoardFilters(filters: BoardFilters): boolean {
  return (
    filters.includeTags.length > 0 ||
    filters.excludeTags.length > 0 ||
    filters.untaggedOnly ||
    filters.tiers.length > 0 ||
    filters.dependency !== "all"
  );
}

/** Every tag in use, normalized and sorted — the options a filter can offer. */
export function collectTagVocabulary(specs: Spec[]): string[] {
  const vocabulary = new Set<string>();
  for (const spec of specs) {
    for (const tag of specTags(spec)) vocabulary.add(tag);
  }
  return [...vocabulary].sort();
}

export function toggleValue(values: string[], value: string): string[] {
  const normalized = normalize(value);
  if (!normalized) return values;
  return values.includes(normalized)
    ? values.filter((entry) => entry !== normalized)
    : [...values, normalized];
}
