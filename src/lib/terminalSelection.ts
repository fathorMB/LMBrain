import type { MouseTrackingMode } from "./terminalWheel";

/**
 * DECRST for every mouse-tracking mode (X10, VT200, button-event, any-event).
 * Written to xterm locally — never to the PTY — so the harness keeps believing
 * its tracking is on while ordinary drag selection works. Encoding modes
 * (SGR/UTF/urxvt) are left untouched: they are inert without tracking and
 * resetting them could desynchronize the harness's report parsing.
 */
export const SUSPEND_MOUSE_TRACKING = "\u001b[?1003l\u001b[?1002l\u001b[?1000l\u001b[?9l";

/**
 * DECSET restoring exactly one previously observed tracking mode; empty when
 * tracking was already off.
 */
export function restoreMouseTracking(mode: MouseTrackingMode): string {
  switch (mode) {
    case "x10":
      return "\u001b[?9h";
    case "vt200":
      return "\u001b[?1000h";
    case "drag":
      return "\u001b[?1002h";
    case "any":
      return "\u001b[?1003h";
    default:
      return "";
  }
}

/** Structural subset of xterm's Terminal needed to read the visible viewport. */
export interface ViewportReader {
  rows: number;
  buffer: {
    active: {
      viewportY: number;
      getLine(y: number): { translateToString(trimRight?: boolean): string } | undefined;
    };
  };
}

/**
 * Serialize the currently visible viewport (not the scrollback, not the full
 * conversation) as plain text with trailing blank lines removed.
 */
export function visibleViewportText(term: ViewportReader): string {
  const lines: string[] = [];
  for (let row = 0; row < term.rows; row += 1) {
    const line = term.buffer.active.getLine(term.buffer.active.viewportY + row);
    lines.push(line ? line.translateToString(true) : "");
  }
  while (lines.length > 0 && lines[lines.length - 1].trim() === "") {
    lines.pop();
  }
  return lines.join("\n");
}
