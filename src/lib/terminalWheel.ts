import type { AgentHost } from "../types";

export type TerminalBufferType = "normal" | "alternate";

/** Mouse tracking mode reported by xterm's modes API at event time. */
export type MouseTrackingMode = "none" | "x10" | "vt200" | "drag" | "any";

/**
 * What the terminal should do with a scroll gesture, decided at event time
 * from the active buffer and mouse-tracking state instead of a per-host
 * assumption:
 * - local: use xterm's own scrollback.
 * - delegate: let xterm handle the event natively (mouse reports when the
 *   TUI tracks the mouse, alternate-screen arrow emulation otherwise).
 * - input: send a documented harness key binding to the PTY.
 * - unsupported: no reliable mapping exists; surface the hint visibly
 *   instead of swallowing the gesture.
 */
export type TerminalScrollAction =
  | { kind: "local" }
  | { kind: "delegate" }
  | { kind: "input"; data: string }
  | { kind: "unsupported"; hint: string };

const PAGE_UP = "\u001b[5~";
const PAGE_DOWN = "\u001b[6~";
const END_KEY = "\u001b[F";
// OpenCode default bindings. Under ConPTY these arrive reliably as ESC + the
// corresponding control byte, unlike PageUp CSI sequences in its embedded
// OpenTUI path.
const OPENCODE_LINE_UP = "\u001b\u0019"; // Ctrl+Alt+Y
const OPENCODE_LINE_DOWN = "\u001b\u0005"; // Ctrl+Alt+E
const OPENCODE_PAGE_UP = "\u001b\u0002"; // Ctrl+Alt+B
const OPENCODE_PAGE_DOWN = "\u001b\u0006"; // Ctrl+Alt+F
export const OPENCODE_SCROLL_TO_BOTTOM = "\u001b\u0007"; // Ctrl+Alt+G

const CLAUDE_HINT =
  "Claude Code owns this view; scroll with the mouse wheel or its in-app keys.";
const UNKNOWN_HINT = "This TUI owns its scrollback; use its own keys.";

export function terminalWheelRows(deltaY: number) {
  return Math.max(1, Math.ceil(Math.abs(deltaY) / 36));
}

/** Cap a single wheel frame to avoid flooding the PTY on high-res devices. */
function opencodeWheelInput(direction: -1 | 1, rows: number) {
  const sequence = direction < 0 ? OPENCODE_LINE_UP : OPENCODE_LINE_DOWN;
  return sequence.repeat(Math.max(1, Math.min(rows, 6)));
}

export function terminalWheelAction(
  host: AgentHost,
  bufferType: TerminalBufferType,
  mouseTracking: MouseTrackingMode,
  direction: -1 | 1,
  rows: number
): TerminalScrollAction {
  if (bufferType === "normal") {
    return { kind: "local" };
  }
  // A full-screen TUI that tracks the mouse gets real wheel reports through
  // xterm; synthetic key sequences would fight its own handling.
  if (mouseTracking !== "none") {
    return { kind: "delegate" };
  }
  switch (host) {
    case "pi":
      return { kind: "input", data: direction < 0 ? PAGE_UP : PAGE_DOWN };
    case "opencode":
      return { kind: "input", data: opencodeWheelInput(direction, rows) };
    case "codex":
      // Codex relies on xterm's alternate-screen arrow emulation.
      return { kind: "delegate" };
    case "claude":
      // Claude Code normally tracks the mouse; without tracking there is no
      // reliable wheel mapping, so degrade visibly instead of guessing.
      return { kind: "unsupported", hint: CLAUDE_HINT };
    default:
      return { kind: "unsupported", hint: UNKNOWN_HINT };
  }
}

export function terminalPageAction(
  host: AgentHost,
  bufferType: TerminalBufferType,
  direction: -1 | 1
): TerminalScrollAction {
  if (bufferType === "normal") {
    return { kind: "local" };
  }
  switch (host) {
    case "pi":
    case "codex":
      return { kind: "input", data: direction < 0 ? PAGE_UP : PAGE_DOWN };
    case "opencode":
      return {
        kind: "input",
        data: direction < 0 ? OPENCODE_PAGE_UP : OPENCODE_PAGE_DOWN,
      };
    case "claude":
      return { kind: "unsupported", hint: CLAUDE_HINT };
    default:
      return { kind: "unsupported", hint: UNKNOWN_HINT };
  }
}

export function terminalBottomAction(
  host: AgentHost,
  bufferType: TerminalBufferType
): TerminalScrollAction {
  if (bufferType === "normal") {
    return { kind: "local" };
  }
  switch (host) {
    case "pi":
    case "codex":
      // End is the portable TUI fallback; applications may map it to the
      // latest item while Page Down remains available for increments.
      return { kind: "input", data: END_KEY };
    case "opencode":
      return { kind: "input", data: OPENCODE_SCROLL_TO_BOTTOM };
    case "claude":
      return { kind: "unsupported", hint: CLAUDE_HINT };
    default:
      return { kind: "unsupported", hint: UNKNOWN_HINT };
  }
}
