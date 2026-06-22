import { describe, it, expect, vi } from "vitest";
import { render } from "@testing-library/react";
import { MarkdownRenderer } from "../lib/markdown";

describe("MarkdownRenderer integration", () => {
  it("renders resolved [[wikilink]] as a clickable navigation control", () => {
    const handler = vi.fn();
    const resolved = new Set(["adr-001"]);
    const { container } = render(
      <MarkdownRenderer
        content="See [[ADR-001]]."
        onWikilinkClick={handler}
        resolvedTargets={resolved}
      />
    );
    // Resolved wikilink: rendered as span with dashed border-bottom
    const spans = container.querySelectorAll("span");
    const resolvedSpan = Array.from(spans).find(
      (s) => s.style.borderBottom && s.style.borderBottom.includes("dashed")
    );
    expect(resolvedSpan).toBeDefined();
    expect(resolvedSpan!.textContent).toContain("ADR-001");
    // Should be focusable (tabIndex 0)
    expect(resolvedSpan!.getAttribute("tabindex")).toBe("0");
    // Should have role="button"
    expect(resolvedSpan!.getAttribute("role")).toBe("button");
    // Click invokes handler
    resolvedSpan!.click();
    expect(handler).toHaveBeenCalledWith("ADR-001");
  });

  it("renders unresolved [[wikilink]] as muted non-interactive text", () => {
    const handler = vi.fn();
    const resolved = new Set<string>(); // empty — nothing is resolved
    const { container } = render(
      <MarkdownRenderer
        content="See [[MISSING-ADR]]."
        onWikilinkClick={handler}
        resolvedTargets={resolved}
      />
    );
    // Unresolved wikilink: rendered as span without dashed border
    const spans = container.querySelectorAll("span");
    const unresolvedSpan = Array.from(spans).find(
      (s) => s.style.cursor === "default"
    );
    expect(unresolvedSpan).toBeDefined();
    expect(unresolvedSpan!.textContent).toContain("MISSING-ADR");
    // Should NOT be focusable
    expect(unresolvedSpan!.getAttribute("tabindex")).toBeNull();
    // Should NOT have role="button"
    expect(unresolvedSpan!.getAttribute("role")).toBeNull();
    // Should have muted color
    expect(unresolvedSpan!.style.color).toBe("rgb(108, 102, 113)");
    // Click should NOT invoke handler
    unresolvedSpan!.click();
    expect(handler).not.toHaveBeenCalled();
  });

  it("renders [[wikilink|display text]] with custom display", () => {
    const handler = vi.fn();
    const resolved = new Set(["spec-001"]);
    const { container } = render(
      <MarkdownRenderer
        content="See [[SPEC-001|the specification]]."
        onWikilinkClick={handler}
        resolvedTargets={resolved}
      />
    );
    const spans = container.querySelectorAll("span");
    const resolvedSpan = Array.from(spans).find(
      (s) => s.style.borderBottom && s.style.borderBottom.includes("dashed")
    );
    expect(resolvedSpan).toBeDefined();
    expect(resolvedSpan!.textContent).toContain("the specification");
    resolvedSpan!.click();
    expect(handler).toHaveBeenCalledWith("SPEC-001");
  });

  it("renders multiple wikilinks with mixed resolution", () => {
    const handler = vi.fn();
    const resolved = new Set(["adr-001"]);
    const { container } = render(
      <MarkdownRenderer
        content="Links: [[ADR-001]] and [[UNKNOWN-REF]]."
        onWikilinkClick={handler}
        resolvedTargets={resolved}
      />
    );
    const spans = container.querySelectorAll("span");
    const resolvedSpans = Array.from(spans).filter(
      (s) => s.style.borderBottom && s.style.borderBottom.includes("dashed")
    );
    const unresolvedSpans = Array.from(spans).filter(
      (s) => s.style.cursor === "default"
    );
    expect(resolvedSpans.length).toBe(1);
    expect(unresolvedSpans.length).toBe(1);
    expect(resolvedSpans[0].textContent).toContain("ADR-001");
    expect(unresolvedSpans[0].textContent).toContain("UNKNOWN-REF");
  });

  it("does not throw when clicking wikilink without handler", () => {
    const resolved = new Set(["adr-001"]);
    const { container } = render(
      <MarkdownRenderer content="See [[ADR-001]]." resolvedTargets={resolved} />
    );
    const spans = container.querySelectorAll("span");
    const resolvedSpan = Array.from(spans).find(
      (s) => s.style.borderBottom && s.style.borderBottom.includes("dashed")
    );
    expect(resolvedSpan).toBeDefined();
    expect(() => resolvedSpan!.click()).not.toThrow();
  });

  it("renders regular markdown links as external links", () => {
    const { container } = render(
      <MarkdownRenderer content="See [example](https://example.com)." />
    );
    const links = container.querySelectorAll("a");
    expect(links.length).toBeGreaterThan(0);
    const exampleLink = Array.from(links).find((l) =>
      l.textContent?.includes("example")
    );
    expect(exampleLink).toBeDefined();
    expect(exampleLink!.getAttribute("href")).toBe("https://example.com");
  });

  it("renders inline code", () => {
    const { container } = render(<MarkdownRenderer content="Use `code` here." />);
    const codes = container.querySelectorAll("code");
    expect(codes.length).toBeGreaterThan(0);
  });

  it("renders blockquotes", () => {
    const { container } = render(<MarkdownRenderer content="> A quote" />);
    const divs = container.querySelectorAll("div");
    const quoteDiv = Array.from(divs).find(
      (d) => d.style.borderLeft && d.textContent?.includes("A quote")
    );
    expect(quoteDiv).toBeDefined();
  });
});
