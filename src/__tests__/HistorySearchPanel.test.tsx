import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { HistorySearchPanel } from "../components/Sessions/HistorySearchPanel";

const mockTranscriptData = "\u001b[31mError in compiler\u001b[0m\nLine 2: Success\nLine 3: Warnings";

vi.mock("../lib/commands", () => ({
  sessionGetTranscript: vi.fn(async () => mockTranscriptData),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => {
    return () => {};
  }),
}));

describe("HistorySearchPanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads and displays stripped ANSI logs from transcript", async () => {
    render(<HistorySearchPanel sessionId="session-1" onClose={vi.fn()} />);

    // Loader is active
    expect(screen.getByText("Loading transcript...")).toBeDefined();

    // Strips ANSI and displays clean lines
    await waitFor(() => {
      expect(screen.getByText("Error in compiler")).toBeDefined();
    });

    expect(screen.getByText("Line 2: Success")).toBeDefined();
    expect(screen.getByText("Line 3: Warnings")).toBeDefined();
  });

  it("filters lines based on query search input", async () => {
    render(<HistorySearchPanel sessionId="session-1" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Error in compiler")).toBeDefined();
    });

    const searchInput = screen.getByPlaceholderText("Search logs...");
    fireEvent.change(searchInput, { target: { value: "Success" } });

    // Renders matching line
    expect(screen.getAllByText((_, node) => node?.textContent === "Line 2: Success").length).toBeGreaterThan(0);
    
    // Non-matching lines should not be rendered
    expect(screen.queryByText("Error in compiler")).toBeNull();
    expect(screen.queryByText("Line 3: Warnings")).toBeNull();
  });

  it("allows selecting lines and copying them to the clipboard", async () => {
    const mockClipboardWrite = vi.fn();
    Object.assign(navigator, {
      clipboard: {
        writeText: mockClipboardWrite,
      },
    });

    render(<HistorySearchPanel sessionId="session-1" onClose={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText("Line 2: Success")).toBeDefined();
    });

    // Select line 2 and line 3 by clicking them
    fireEvent.click(screen.getByText("Line 2: Success"));
    fireEvent.click(screen.getByText("Line 3: Warnings"));

    // Shows selection panel
    expect(screen.getByText("2 lines selected")).toBeDefined();

    // Copy selected lines
    const copyButton = screen.getByText("Copy Selected");
    fireEvent.click(copyButton);

    expect(mockClipboardWrite).toHaveBeenCalledWith("Line 2: Success\nLine 3: Warnings");
  });
});
