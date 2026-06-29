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
  convertFileSrc: (path: string) => `asset://localhost/${path}`,
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
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: vi.fn(() => "blob:preview-url"),
      revokeObjectURL: vi.fn(),
    });
    vi.mocked(commands.readDesignMockupHtml).mockResolvedValue({
      path: mockup.entry_path,
      content: "<!doctype html><html><body>Checkout</body></html>",
    });
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
    expect(frame.getAttribute("src")).toBe("blob:preview-url");
    expect(commands.readDesignMockupHtml).toHaveBeenCalledWith(mockup.entry_path);
  });
});
