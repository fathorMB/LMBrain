import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { DesignView } from "../components/Design/DesignView";
import * as commands from "../lib/commands";
import type { DesignMockup } from "../types";

vi.mock("../lib/commands", () => ({
  getDesignMockups: vi.fn(),
  readDesignMockupHtml: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string, protocol = "asset") => `${protocol}://localhost/${path}`,
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
    vi.mocked(commands.readDesignMockupHtml).mockResolvedValue({
      path: "",
      content: "",
    });

    render(<DesignView />);

    await waitFor(() => expect(screen.getByText("No design mockups")).toBeDefined());
    expect(screen.getByText(/.lmbrain\/design/)).toBeDefined();
  });

  it("renders mockup metadata and preview frame", async () => {
    vi.mocked(commands.getDesignMockups).mockResolvedValue([mockup]);
    vi.mocked(commands.readDesignMockupHtml).mockResolvedValue({
      path: "E:/workspace/.lmbrain/design/checkout-flow/index.html",
      content: '<!doctype html><html><head><title>Checkout</title></head><body><script src="assets/app.js"></script></body></html>',
    });

    render(<DesignView />);

    await waitFor(() => expect(screen.getAllByText("Checkout Flow").length).toBeGreaterThan(0));
    expect(screen.getByText("Responsive checkout mockup.")).toBeDefined();
    const frame = await screen.findByTitle("Design mockup preview");
    expect(commands.readDesignMockupHtml).toHaveBeenCalledWith(mockup.entry_path);
    expect(frame.getAttribute("srcdoc")).toContain(
      '<base href="lmbrain-design://localhost/.lmbrain/design/checkout-flow/">'
    );
    expect(frame.getAttribute("srcdoc")).toContain('script src="assets/app.js"');
  });
});
