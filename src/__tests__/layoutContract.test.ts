import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Layout contract (ISSUE-50-PLAN.md §5). There is no screenshot-based visual
 * regression tooling in this repository, so these assertions are what stop a
 * page from silently regressing to a hand-rolled container, an off-scale font
 * size, or a hardcoded low-contrast colour.
 */

const ROOT = join(process.cwd(), "src");

function source(relativePath: string): string {
  return readFileSync(join(ROOT, relativePath), "utf8");
}

const DENSE_PAGES = [
  "components/Decisions/DecisionsList.tsx",
  "components/Reviews/ReviewsList.tsx",
  "components/Debts/DebtsView.tsx",
  "components/Feedback/FeedbackView.tsx",
  "components/Agents/AgentsView.tsx",
  "components/Agents/McpView.tsx",
  "components/Skills/SkillsView.tsx",
  "components/Insights/InsightsView.tsx",
  "components/Operations/OperationsView.tsx",
  "components/Roadmap/RoadmapView.tsx",
  "components/Harnesses/HarnessesView.tsx",
  "components/Pulse/ProjectPulse.tsx",
];

const READING_PAGES = ["components/Spec/SpecDetail.tsx"];

/** Full-bleed surfaces: panes own their scrolling, so they keep their own shell. */
const FULL_BLEED_PAGES = [
  "components/Taskboard/TaskboardView.tsx",
  "components/Sessions/SessionsView.tsx",
  "components/Wiki/WikiView.tsx",
  "components/Design/DesignView.tsx",
  "components/Repository/RepositoryView.tsx",
  "components/Settings/SettingsView.tsx",
];

const ALL_PAGES = [...DENSE_PAGES, ...READING_PAGES, ...FULL_BLEED_PAGES];

describe("page archetypes", () => {
  it("routes the shared list view through the dense archetype", () => {
    // Pages that delegate their shell to ArtifactListView inherit the
    // archetype from here, so this is the assertion that keeps them honest.
    expect(source("components/Shared/ArtifactListView.tsx")).toContain(
      '<PageShell archetype="dense">',
    );
  });

  it.each(DENSE_PAGES)("%s renders through the dense archetype", (path) => {
    const text = source(path);
    const rendersDenseShell = text.includes('<PageShell archetype="dense">');
    const delegatesToSharedListView = text.includes("<ArtifactListView");
    expect(rendersDenseShell || delegatesToSharedListView).toBe(true);
  });

  it.each(READING_PAGES)("%s renders through the reading archetype", (path) => {
    expect(source(path)).toContain('<PageShell archetype="reading">');
  });

  it.each(ALL_PAGES)("%s does not centre a hardcoded content column", (path) => {
    // Numeric maxWidth on a centred container is the pattern the layout system
    // replaced; widths now come from --page-reading / --page-wide.
    expect(source(path)).not.toMatch(/maxWidth: \d+,\s*margin: "0 auto"/);
  });
});

describe("shared scales", () => {
  it.each(ALL_PAGES)("%s carries no half-pixel font sizes", (path) => {
    expect(source(path)).not.toMatch(/fontSize: \d+\.\d/);
  });

  it.each(ALL_PAGES)("%s does not hardcode the text-colour tokens", (path) => {
    const text = source(path);
    for (const legacy of ["#6c6671", "#9a949f", "#56525b"]) {
      expect(text).not.toContain(legacy);
    }
  });
});

describe("global tokens", () => {
  const css = source("styles/global.css");

  it("defines the spacing, radius, type and page-geometry scales", () => {
    for (const token of [
      "--space-1:",
      "--space-7:",
      "--radius-sm:",
      "--radius-pill:",
      "--text-2xs:",
      "--text-2xl:",
      "--page-reading:",
      "--page-wide:",
      "--page-wide-xl:",
      "--page-gutter:",
    ]) {
      expect(css).toContain(token);
    }
  });

  it("keeps a shared focus-visible ring at zero specificity", () => {
    expect(css).toMatch(/:where\([^)]*\):focus-visible/);
    expect(css).toContain("outline: 2px solid var(--accent-light)");
  });

  it("uses the raised-contrast text colours", () => {
    expect(css).toContain("--text-tertiary: #8b8592");
    expect(css).toContain("--text-muted: #827d8b");
    expect(css).not.toContain("--text-tertiary: #6c6671");
  });
});

describe("responsive layout rules", () => {
  const css = source("components/Shared/layout.css");

  it("caps the dense archetype and relaxes it on very wide viewports", () => {
    expect(css).toContain("max-width: var(--page-wide)");
    expect(css).toMatch(/@media \(min-width: 2200px\)[\s\S]*--page-wide-xl/);
  });

  it("flows cards into as many columns as the width allows", () => {
    expect(css).toContain("repeat(auto-fill, minmax(min(var(--lm-card-min, 360px), 100%), 1fr))");
  });
});
