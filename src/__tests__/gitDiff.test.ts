import { describe, expect, it } from "vitest";
import { parseUnifiedDiff } from "../lib/gitDiff";

describe("parseUnifiedDiff", () => {
  it("tracks old and new line numbers through a unified diff hunk", () => {
    const lines = parseUnifiedDiff("@@ -10,2 +10,3 @@\n context\n-removed\n+added\n+second");

    expect(lines.map(({ kind, oldLine, newLine }) => ({ kind, oldLine, newLine }))).toEqual([
      { kind: "hunk", oldLine: null, newLine: null },
      { kind: "context", oldLine: 10, newLine: 10 },
      { kind: "deletion", oldLine: 11, newLine: null },
      { kind: "addition", oldLine: null, newLine: 11 },
      { kind: "addition", oldLine: null, newLine: 12 },
    ]);
  });

  it("classifies patch headers and metadata without assigning line numbers", () => {
    const lines = parseUnifiedDiff("diff --git a/a b/a\n--- a/a\n+++ b/a\n\\ No newline at end of file");
    expect(lines.map((line) => line.kind)).toEqual(["header", "header", "header", "meta"]);
    expect(lines.every((line) => line.oldLine === null && line.newLine === null)).toBe(true);
  });
});
