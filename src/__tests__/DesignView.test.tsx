import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { DesignView, designPreviewUrl } from "../components/Design/DesignView";
import * as commands from "../lib/commands";
import type { DesignMockup } from "../types";

vi.mock("../lib/commands", () => ({
  getDesignMockups: vi.fn(),
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
    vi.stubGlobal("fetch", vi.fn(async () => ({ ok: true })));
  });

  it("renders an empty state when there are no mockups", async () => {
    vi.mocked(commands.getDesignMockups).mockResolvedValue([]);

    render(<DesignView />);

    await waitFor(() => expect(screen.getByText("No design mockups")).toBeDefined());
    expect(screen.getByText(/.lmbrain\/design/)).toBeDefined();
  });

  it("renders mockup metadata and preview frame", async () => {
    vi.mocked(commands.getDesignMockups).mockResolvedValue([mockup]);

    render(<DesignView />);

    await waitFor(() => expect(screen.getAllByText("Checkout Flow").length).toBeGreaterThan(0));
    expect(screen.getByText("Responsive checkout mockup.")).toBeDefined();
    const frame = await screen.findByTitle("Design mockup preview");
    expect(frame.getAttribute("src")).toBe(designPreviewUrl(mockup.entry_path));
    expect(frame.hasAttribute("srcdoc")).toBe(false);
  });

  it("shows the preview error state when the protocol handler rejects the asset", async () => {
    vi.mocked(commands.getDesignMockups).mockResolvedValue([mockup]);
    vi.stubGlobal("fetch", vi.fn(async () => ({ ok: false })));

    render(<DesignView />);

    await waitFor(() =>
      expect(screen.getByText("Preview unavailable for this design mockup.")).toBeDefined()
    );
  });
});

describe("designPreviewUrl", () => {
  it("uses the http bridge form on Windows", () => {
    expect(
      designPreviewUrl(
        ".lmbrain\\design\\checkout-flow\\index.html",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
      )
    ).toBe("http://lmbrain-design.localhost/.lmbrain/design/checkout-flow/index.html");
  });

  it("uses the native scheme form on Linux and macOS", () => {
    for (const ua of [
      "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15",
      "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15",
    ]) {
      expect(designPreviewUrl(".lmbrain/design/checkout-flow/index.html", ua)).toBe(
        "lmbrain-design://localhost/.lmbrain/design/checkout-flow/index.html"
      );
    }
  });
});
