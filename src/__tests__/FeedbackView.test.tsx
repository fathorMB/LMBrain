import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FeedbackView } from "../components/Feedback/FeedbackView";
import { getKitFeedback, saveTextFile } from "../lib/commands";
import { save } from "@tauri-apps/plugin-dialog";
import type { KitFeedbackReport } from "../types";

vi.mock("../lib/commands", () => ({
  getKitFeedback: vi.fn(),
  saveTextFile: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
}));

const report: KitFeedbackReport = {
  schema_version: "1",
  path: ".lmbrain/reports/lmbrain-kit-feedback.md",
  updated: "2026-07-31",
  total: 3,
  counts_by_category: { usability: 2, compatibility: 1 },
  counts_by_severity: { high: 1, medium: 2 },
  notes: [
    {
      id: "KIT-NOTE-001", timestamp: "2026-07-30", lmbrain_version: "3.1.4", category: "usability", severity: "medium",
      summary: "Current note", observed_behavior: "Current behavior", expected_behavior: "Expected behavior", impact: "Medium impact",
      evidence: "Evidence", workaround: null, suggested_improvement: null, related_note: null, actor: "AGENT-LEAD",
    },
    {
      id: "KIT-NOTE-002", timestamp: "2026-07-29", lmbrain_version: "3.1.3", category: "compatibility", severity: "high",
      summary: "Older note", observed_behavior: "Older behavior", expected_behavior: "Expected behavior", impact: "High impact",
      evidence: "Evidence", workaround: null, suggested_improvement: null, related_note: null, actor: "AGENT-LEAD",
    },
    {
      id: "KIT-NOTE-003", timestamp: "2026-07-28", lmbrain_version: "3.1.4", category: "usability", severity: "medium",
      summary: "Second current note", observed_behavior: "Current behavior", expected_behavior: "Expected behavior", impact: "Medium impact",
      evidence: "Evidence", workaround: null, suggested_improvement: null, related_note: null, actor: "AGENT-LEAD",
    },
  ],
};

describe("FeedbackView", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  it("lists distinct versions in deterministic order and filters notes by version", async () => {
    vi.mocked(getKitFeedback).mockResolvedValue(report);
    render(<FeedbackView />);

    await waitFor(() => expect(screen.getByText("Current note")).toBeDefined());
    const versionFilter = screen.getByLabelText("Feedback version") as HTMLSelectElement;
    expect(Array.from(versionFilter.options).map((option) => option.textContent)).toEqual(["All versions", "v3.1.4", "v3.1.3"]);

    fireEvent.change(versionFilter, { target: { value: "3.1.3" } });
    expect(screen.getByText("Older note")).toBeDefined();
    expect(screen.queryByText("Current note")).toBeNull();
    expect(screen.queryByText("Second current note")).toBeNull();
  });

  it("composes version filtering with severity, category, and search", async () => {
    vi.mocked(getKitFeedback).mockResolvedValue(report);
    render(<FeedbackView />);

    await waitFor(() => expect(screen.getByText("Current note")).toBeDefined());
    fireEvent.change(screen.getByLabelText("Feedback version"), { target: { value: "3.1.4" } });
    fireEvent.change(screen.getByLabelText("Feedback category"), { target: { value: "usability" } });
    fireEvent.change(screen.getByLabelText("Search feedback"), { target: { value: "second" } });

    expect(screen.getByText("Second current note")).toBeDefined();
    expect(screen.queryByText("Current note")).toBeNull();
    expect(screen.queryByText("Older note")).toBeNull();
  });

  it("offers all-items and version-specific read-only JSON exports", async () => {
    vi.mocked(getKitFeedback).mockResolvedValue(report);
    vi.mocked(save).mockResolvedValue("C:/Exports/lmbrain-kit-feedback-v3.1.4.json");
    vi.mocked(saveTextFile).mockResolvedValue(undefined);
    render(<FeedbackView />);

    await waitFor(() => expect(screen.getByText("Current note")).toBeDefined());
    fireEvent.click(screen.getByRole("button", { name: "Export feedback" }));
    expect(screen.getByRole("button", { name: "Download all items" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Download v3.1.4" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Download v3.1.3" })).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Download v3.1.4" }));
    await waitFor(() => expect(saveTextFile).toHaveBeenCalledTimes(1));
    expect(save).toHaveBeenCalledWith({
      defaultPath: "lmbrain-kit-feedback-v3.1.4.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    expect(saveTextFile.mock.calls[0][0]).toBe("C:/Exports/lmbrain-kit-feedback-v3.1.4.json");
    expect(saveTextFile.mock.calls[0][1]).toContain('"lmbrain_version": "3.1.4"');
    expect(screen.queryByRole("button", { name: "Download all items" })).toBeNull();
  });
});
