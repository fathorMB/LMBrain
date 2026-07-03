import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { DesignView } from "../components/Design/DesignView";
import * as commands from "../lib/commands";
import type { DesignMockup } from "../types";

vi.mock("../lib/commands", () => ({
  getDesignMockups: vi.fn(),
  readDesignMockupPreviewHtml: vi.fn(),
}));

const mockup: DesignMockup = {
  id: "checkout-flow",
  name: "checkout-flow",
  path: ".lmbrain/design/checkout-flow",
  entry_path: ".lmbrain/design/checkout-flow/index.html",
  kind: "package",
  modified: "0d 0h 1m ago",
  size: 2048,
  summary: "Responsive checkout mockup.",
  manifest_title: "Checkout Flow",
  manifest_description: "Responsive checkout mockup.",
  has_manifest: true,
  has_readme: true,
};

describe("DesignView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders an empty state when there are no mockups", async () => {
    vi.mocked(commands.getDesignMockups).mockResolvedValue([]);
    vi.mocked(commands.readDesignMockupPreviewHtml).mockResolvedValue({
      path: "",
      content: "",
    });

    render(<DesignView />);

    await waitFor(() => expect(screen.getByText("No design mockups")).toBeDefined());
    expect(screen.getByText(/.lmbrain\/design/)).toBeDefined();
  });

  it("renders mockup metadata and preview frame", async () => {
    vi.mocked(commands.getDesignMockups).mockResolvedValue([mockup]);
    vi.mocked(commands.readDesignMockupPreviewHtml).mockResolvedValue({
      path: "E:/workspace/.lmbrain/design/checkout-flow/index.html",
      content:
        '<!doctype html><html><head><style data-lmbrain-inline="assets/design-system.css">body{color:red}</style></head><body><script data-lmbrain-inline="assets/app.js">window.ready=true</script></body></html>',
    });

    render(<DesignView />);

    await waitFor(() => expect(screen.getAllByText("Checkout Flow").length).toBeGreaterThan(0));
    expect(screen.getByText("Responsive checkout mockup.")).toBeDefined();
    const frame = await screen.findByTitle("Design mockup preview");
    expect(commands.readDesignMockupPreviewHtml).toHaveBeenCalledWith(mockup.entry_path);
    expect(frame.getAttribute("srcdoc")).toContain('data-lmbrain-inline="assets/design-system.css"');
    expect(frame.getAttribute("srcdoc")).toContain('data-lmbrain-inline="assets/app.js"');
    expect(frame.hasAttribute("src")).toBe(false);
  });
});
