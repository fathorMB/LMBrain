import { describe, it, expect } from "vitest";
import { preprocessWikilinksToMarkdown } from "../lib/remark-wikilinks";

describe("preprocessWikilinksToMarkdown", () => {
  it("converts simple [[wikilink]] to markdown link", () => {
    const result = preprocessWikilinksToMarkdown("See [[ADR-001]] for details.");
    expect(result).toBe("See [ADR-001](#wikilink:ADR-001) for details.");
  });

  it("converts [[wikilink|display text]] with custom display", () => {
    const result = preprocessWikilinksToMarkdown("See [[SPEC-001|the spec]].");
    expect(result).toBe("See [the spec](#wikilink:SPEC-001).");
  });

  it("handles multiple wikilinks", () => {
    const result = preprocessWikilinksToMarkdown(
      "Links: [[ADR-001]] and [[SPEC-002]]."
    );
    expect(result).toBe(
      "Links: [ADR-001](#wikilink:ADR-001) and [SPEC-002](#wikilink:SPEC-002)."
    );
  });

  it("handles text without wikilinks", () => {
    const result = preprocessWikilinksToMarkdown("No wikilinks here.");
    expect(result).toBe("No wikilinks here.");
  });

  it("handles empty string", () => {
    const result = preprocessWikilinksToMarkdown("");
    expect(result).toBe("");
  });

  it("encodes special characters in target", () => {
    const result = preprocessWikilinksToMarkdown("See [[ADR-001|my link]].");
    expect(result).toContain("#wikilink:ADR-001");
    expect(result).toContain("my link");
  });

  it("produces valid markdown that can be parsed", () => {
    const result = preprocessWikilinksToMarkdown("See [[TARGET]].");
    expect(result).toMatch(/\[TARGET\]\(#wikilink:TARGET\)/);
  });
});
