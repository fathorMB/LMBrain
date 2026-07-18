import { describe, expect, it } from "vitest";
import { terminalWheelAction, terminalWheelRows } from "../lib/terminalWheel";
import type { AgentHost } from "../types";

const HOSTS: AgentHost[] = ["claude", "codex", "pi", "opencode"];

describe("terminal wheel policy", () => {
  it("converts wheel delta to at least one scrollback row", () => {
    expect(terminalWheelRows(0)).toBe(1);
    expect(terminalWheelRows(72)).toBe(2);
    expect(terminalWheelRows(-73)).toBe(3);
  });

  it("delegates normal-buffer wheel events to xterm's native scrollback", () => {
    for (const host of HOSTS) {
      expect(terminalWheelAction(host, "normal", "none", 1, 1)).toEqual({ kind: "delegate" });
      expect(terminalWheelAction(host, "normal", "any", -1, 1)).toEqual({ kind: "delegate" });
    }
  });

  it("delegates alternate-buffer wheel to xterm whenever the TUI tracks the mouse", () => {
    for (const host of HOSTS) {
      for (const tracking of ["x10", "vt200", "drag", "any"] as const) {
        expect(terminalWheelAction(host, "alternate", tracking, 1, 1)).toEqual({
          kind: "delegate",
        });
      }
    }
  });

  it("maps untracked alternate-buffer wheel per documented host bindings", () => {
    expect(terminalWheelAction("pi", "alternate", "none", -1, 1)).toEqual({
      kind: "input",
      data: "\u001b[5~",
    });
    expect(terminalWheelAction("pi", "alternate", "none", 1, 1)).toEqual({
      kind: "input",
      data: "\u001b[6~",
    });
    expect(terminalWheelAction("opencode", "alternate", "none", -1, 2)).toEqual({
      kind: "input",
      data: "\u001b\u0019\u001b\u0019",
    });
    expect(terminalWheelAction("opencode", "alternate", "none", 1, 99)).toEqual({
      kind: "input",
      data: "\u001b\u0005".repeat(6),
    });
    expect(terminalWheelAction("codex", "alternate", "none", 1, 1)).toEqual({
      kind: "delegate",
    });
  });

  it("degrades visibly instead of swallowing wheel input without a mapping", () => {
    const claude = terminalWheelAction("claude", "alternate", "none", 1, 1);
    expect(claude.kind).toBe("unsupported");
    if (claude.kind === "unsupported") expect(claude.hint).toContain("Claude Code");
    const unknown = terminalWheelAction("future-tui" as AgentHost, "alternate", "none", 1, 1);
    expect(unknown.kind).toBe("unsupported");
  });
});
