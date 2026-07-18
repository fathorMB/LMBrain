export type DiffLineKind = "header" | "hunk" | "addition" | "deletion" | "context" | "meta";

export interface ParsedDiffLine {
  content: string;
  kind: DiffLineKind;
  oldLine: number | null;
  newLine: number | null;
}

const HUNK_HEADER = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/;

export function parseUnifiedDiff(diff: string): ParsedDiffLine[] {
  let oldLine = 0;
  let newLine = 0;

  return diff.split("\n").map((content) => {
    const hunk = HUNK_HEADER.exec(content);
    if (hunk) {
      oldLine = Number(hunk[1]);
      newLine = Number(hunk[2]);
      return { content, kind: "hunk", oldLine: null, newLine: null };
    }
    if (content.startsWith("diff --git ") || content.startsWith("index ") || content.startsWith("--- ") || content.startsWith("+++ ")) {
      return { content, kind: "header", oldLine: null, newLine: null };
    }
    if (content.startsWith("+") && !content.startsWith("+++")) {
      const line = { content, kind: "addition" as const, oldLine: null, newLine };
      newLine += 1;
      return line;
    }
    if (content.startsWith("-") && !content.startsWith("---")) {
      const line = { content, kind: "deletion" as const, oldLine, newLine: null };
      oldLine += 1;
      return line;
    }
    if (content.startsWith(" ")) {
      const line = { content, kind: "context" as const, oldLine, newLine };
      oldLine += 1;
      newLine += 1;
      return line;
    }
    return { content, kind: "meta", oldLine: null, newLine: null };
  });
}
