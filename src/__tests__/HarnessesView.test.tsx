import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { HarnessesView } from "../components/Harnesses/HarnessesView";
import { probeHarnesses, updateHarness } from "../lib/commands";
import type { HarnessStatus } from "../types";

const workspace = vi.hoisted(() => ({ state: { sessions: [] as Array<{ host: "claude" | "codex" | "pi" | "opencode"; status: "running" | "exited"; label: string }> } }));

vi.mock("../hooks/useWorkspace", () => ({ useWorkspace: () => workspace }));
vi.mock("../lib/commands", () => ({ probeHarnesses: vi.fn(), updateHarness: vi.fn() }));

const writeText = vi.fn().mockResolvedValue(undefined);
Object.assign(navigator, { clipboard: { writeText } });

const statuses: HarnessStatus[] = [
  {
    host: "claude",
    label: "Claude Code",
    state: "installed",
    executable: "C:\\Tools\\claude.exe",
    version: "2.1.206",
    detail: null,
    probed_at: "2026-07-11T10:00:00+02:00",
    install_url: "https://example.com/claude",
    install_command: "npm install -g @anthropic-ai/claude-code",
  },
  {
    host: "codex",
    label: "Codex",
    state: "installed",
    executable: "C:\\Tools\\codex.exe",
    version: "0.144.1",
    detail: null,
    probed_at: "2026-07-11T10:00:00+02:00",
    install_url: "https://example.com/codex",
    install_command: "npm install -g @openai/codex",
  },
  {
    host: "pi",
    label: "Pi",
    state: "missing",
    executable: null,
    version: null,
    detail: "Executable not found",
    probed_at: "2026-07-11T10:00:00+02:00",
    install_url: "https://example.com/pi",
    install_command: "npm install -g @earendil-works/pi-coding-agent",
  },
  {
    host: "opencode",
    label: "OpenCode",
    state: "installed",
    executable: "C:\\Tools\\opencode.exe",
    version: "1.15.11",
    detail: null,
    probed_at: "2026-07-11T10:00:00+02:00",
    install_url: "https://opencode.ai/docs/",
    install_command: "npm install -g opencode-ai",
  },
];

describe("HarnessesView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    workspace.state.sessions = [];
    vi.mocked(probeHarnesses).mockResolvedValue(statuses);
  });

  it("shows installed versions, executable paths, and missing-install guidance", async () => {
    render(<HarnessesView />);

    await screen.findByText("2.1.206");
    expect(screen.getByText("0.144.1")).toBeDefined();
    expect(screen.getByText("1.15.11")).toBeDefined();
    expect(screen.getByTitle("C:\\Tools\\claude.exe")).toBeDefined();
    expect(screen.getByText("Not installed")).toBeDefined();

    fireEvent.click(screen.getByRole("button", { name: "Copy install command" }));
    await waitFor(() => expect(writeText).toHaveBeenCalledWith("npm install -g @earendil-works/pi-coding-agent"));
  });

  it("requires confirmation and presents the post-update verified result", async () => {
    vi.mocked(updateHarness).mockResolvedValue({
      host: "claude",
      success: true,
      already_current: false,
      before: statuses[0],
      after: { ...statuses[0], version: "2.1.207" },
      exit_code: 0,
      timed_out: false,
      stdout: "Updated Claude Code",
      stderr: "",
    });
    render(<HarnessesView />);

    await screen.findByText("2.1.206");
    fireEvent.click(screen.getAllByRole("button", { name: "Check & update" })[0]);
    expect(screen.getByRole("dialog")).toBeDefined();
    expect(updateHarness).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Confirm update" }));

    await screen.findByText("2.1.207");
    expect(updateHarness).toHaveBeenCalledWith({ host: "claude", codex_bin: undefined });
    expect(screen.getByText("Update verified")).toBeDefined();
  });

  it("blocks updates while a matching harness session is running", async () => {
    workspace.state.sessions = [{ host: "codex", status: "running", label: "Review work" }];
    render(<HarnessesView />);

    await screen.findByText("0.144.1");
    expect(screen.getByText("Close running session: Review work")).toBeDefined();
    const blockedButton = screen.getByRole("button", { name: "Close sessions first" });
    expect(blockedButton).toHaveProperty("disabled", true);
    expect(updateHarness).not.toHaveBeenCalled();
  });

  it("keeps the current status and exposes backend update failures", async () => {
    vi.mocked(updateHarness).mockRejectedValue("Another harness update is already running for Pi");
    render(<HarnessesView />);

    await screen.findByText("2.1.206");
    fireEvent.click(screen.getAllByRole("button", { name: "Check & update" })[0]);
    fireEvent.click(screen.getByRole("button", { name: "Confirm update" }));

    await screen.findByText("Another harness update is already running for Pi");
    expect(screen.getByText("2.1.206")).toBeDefined();
  });
});
