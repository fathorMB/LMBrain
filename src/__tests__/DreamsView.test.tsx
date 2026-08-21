import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DreamsView } from "../components/Dreams/DreamsView";

const refreshWorkspaceData = vi.fn().mockResolvedValue(undefined);
const dreams = [{
  id: "DREAM-001", title: "A long-form observation", status: "captured",
  classification: "design-debt", confidence: "high", area: "journal",
  related_artifacts: ["SPEC-070"], context_digest: "sha256:abc",
  created: "2026-08-12", updated: "2026-08-13",
  body: "## Rationale\n\n**Readable** long-form content with a [reference](https://example.com).",
  path: ".lmbrain/dreams/captured/DREAM-001.md", malformed: false,
}];

vi.mock("../hooks/useWorkspace", () => ({
  useWorkspace: () => ({ state: { dreams }, refreshWorkspaceData }),
}));
vi.mock("../lib/commands", () => ({ getDreams: vi.fn() }));

describe("DreamsView", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders compact scannable cards and filters them", () => {
    render(<DreamsView />);
    expect(screen.getByRole("button", { name: /Open DREAM-001/ })).toBeDefined();
    expect(screen.getAllByText("design-debt").length).toBeGreaterThan(0);
    expect(screen.getByText("high confidence")).toBeDefined();
    expect(screen.queryByRole("heading", { name: "Rationale" })).toBeNull();
    fireEvent.change(screen.getByLabelText("Dream confidence"), { target: { value: "low" } });
    expect(screen.queryByRole("button", { name: /Open DREAM-001/ })).toBeNull();
    expect(screen.getByText("No dreams match these filters.")).toBeDefined();
  });

  it("opens full Markdown in an accessible focused dialog", () => {
    render(<DreamsView />);
    const opener = screen.getByRole("button", { name: /Open DREAM-001/ });
    opener.focus();
    fireEvent.click(opener);
    const dialog = screen.getByRole("dialog", { name: "A long-form observation" });
    expect(screen.getByRole("heading", { name: "Rationale" })).toBeDefined();
    expect(screen.getByRole("link", { name: "reference" })).toBeDefined();
    expect(screen.getByText("Provenance and suggested disposition")).toBeDefined();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(opener);
    expect(dialog).toBeDefined();
  });

  it("shows the established empty state", () => {
    dreams.splice(0, dreams.length);
    render(<DreamsView />);
    expect(screen.getByText(/No dreams captured yet/)).toBeDefined();
  });
});
