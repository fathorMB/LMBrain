import { describe, expect, it } from "vitest";
import {
  OPENCODE_SCROLL_TO_BOTTOM,
  shouldDelegateTerminalWheel,
  terminalPageAction,
  terminalWheelRows,
  tuiWheelInput,
} from "../lib/terminalWheel";

describe("terminal wheel policy", () => {
  it("delegates alternate-buffer wheel events to xterm for full-screen TUIs", () => {
    expect(
      shouldDelegateTerminalWheel("alternate", { ctrlKey: false, metaKey: false })
    ).toBe(true);
  });

  it("keeps normal-buffer wheel events in local scrollback", () => {
    expect(
      shouldDelegateTerminalWheel("normal", { ctrlKey: false, metaKey: false })
    ).toBe(false);
  });

  it("does not claim modifier-assisted wheel gestures", () => {
    expect(shouldDelegateTerminalWheel("normal", { ctrlKey: true, metaKey: false })).toBe(true);
    expect(shouldDelegateTerminalWheel("normal", { ctrlKey: false, metaKey: true })).toBe(true);
  });

  it("converts wheel delta to at least one scrollback row", () => {
    expect(terminalWheelRows(0)).toBe(1);
    expect(terminalWheelRows(72)).toBe(2);
    expect(terminalWheelRows(-73)).toBe(3);
  });

  it("uses local pages for normal scrollback and PTY page keys for full-screen TUIs", () => {
    expect(terminalPageAction("normal", -1)).toEqual({ kind: "local", pages: -1 });
    expect(terminalPageAction("alternate", -1)).toEqual({ kind: "input", data: "\u001b[5~" });
    expect(terminalPageAction("alternate", 1)).toEqual({ kind: "input", data: "\u001b[6~" });
    expect(terminalPageAction("alternate", -1, "opencode")).toEqual({
      kind: "input",
      data: "\u001b\u0002",
    });
    expect(terminalPageAction("alternate", 1, "opencode")).toEqual({
      kind: "input",
      data: "\u001b\u0006",
    });
  });

  it("maps OpenCode wheel and bottom controls to documented message bindings", () => {
    expect(tuiWheelInput("opencode", -1, 2)).toBe("\u001b\u0019\u001b\u0019");
    expect(tuiWheelInput("opencode", 1, 99)).toBe("\u001b\u0005".repeat(6));
    expect(tuiWheelInput("claude", -1, 1)).toBe("\u001b[5~");
    expect(tuiWheelInput("pi", 1, 1)).toBe("\u001b[6~");
    expect(OPENCODE_SCROLL_TO_BOTTOM).toBe("\u001b\u0007");
  });
});
