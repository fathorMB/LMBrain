import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { sessionAttach, sessionResize, sessionWrite } from "../../lib/commands";
import { HistorySearchPanel } from "./HistorySearchPanel";
import { terminalClipboardAction } from "../../lib/terminalClipboard";
import { terminalWheelAction, terminalWheelRows } from "../../lib/terminalWheel";
import type { MouseTrackingMode } from "../../lib/terminalWheel";
import type { AgentHost } from "../../types";

interface SessionTerminalProps {
  sessionId: string;
  active: boolean;
  host: AgentHost;
}

function trackingMode(term: Terminal): MouseTrackingMode {
  return (term.modes.mouseTrackingMode ?? "none") as MouseTrackingMode;
}

export function SessionTerminal({ sessionId, active, host }: SessionTerminalProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const lastSizeRef = useRef<{ cols: number; rows: number } | null>(null);
  const feedbackTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [feedback, setFeedback] = useState<string | null>(null);
  const [mouseTracking, setMouseTracking] = useState<MouseTrackingMode>("none");
  const [showHistorySearch, setShowHistorySearch] = useState(false);
  const showFeedback = useCallback((message: string) => {
    setFeedback(message);
    if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
    feedbackTimeoutRef.current = setTimeout(() => {
      setFeedback(null);
      feedbackTimeoutRef.current = null;
    }, 3200);
  }, []);

  const copySelection = useCallback(async () => {
    const term = terminalRef.current;
    const selection = term?.getSelection() ?? "";
    if (!selection) {
      showFeedback(
        term && trackingMode(term) !== "none"
          ? "No selection — the TUI captures the mouse: use Shift+drag."
          : "Select terminal text before copying."
      );
      return;
    }
    if (!navigator.clipboard) {
      showFeedback("Clipboard access is unavailable in this WebView.");
      return;
    }
    try {
      await navigator.clipboard.writeText(selection);
      showFeedback("Selection copied.");
    } catch {
      showFeedback("The WebView blocked the clipboard write; try Ctrl+Shift+C.");
    }
  }, [showFeedback]);

  const pasteClipboard = useCallback(async () => {
    const term = terminalRef.current;
    if (!term || !navigator.clipboard) {
      showFeedback("Clipboard access is unavailable in this WebView.");
      return;
    }
    try {
      const text = await navigator.clipboard.readText();
      if (!text) {
        showFeedback("Clipboard is empty.");
        return;
      }
      term.paste(text);
      term.focus();
      showFeedback("Clipboard pasted.");
    } catch {
      showFeedback("Could not read from the clipboard.");
    }
  }, [showFeedback]);

  useEffect(
    () => () => {
      if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
    },
    []
  );

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }

    const term = new Terminal({
      cursorBlink: true,
      allowTransparency: true,
      convertEol: false,
      fontFamily: "var(--font-mono)",
      fontSize: 12.5,
      lineHeight: 1.25,
      scrollback: 5000,
      theme: {
        background: "#120f18",
        foreground: "#efe9f9",
        cursor: "#f7f2ff",
        cursorAccent: "#120f18",
        selectionBackground: "rgba(188, 174, 246, 0.24)",
      },
    });
    const fitAddon = new FitAddon();
    term.loadAddon(fitAddon);
    term.open(container);

    terminalRef.current = term;
    fitAddonRef.current = fitAddon;

    const syncSize = () => {
      const currentTerm = terminalRef.current;
      const currentFitAddon = fitAddonRef.current;
      if (
        !currentTerm ||
        !currentFitAddon ||
        !containerRef.current ||
        containerRef.current.clientWidth === 0 ||
        containerRef.current.clientHeight === 0
      ) {
        return;
      }

      currentFitAddon.fit();
      if (
        !lastSizeRef.current ||
        lastSizeRef.current.cols !== currentTerm.cols ||
        lastSizeRef.current.rows !== currentTerm.rows
      ) {
        lastSizeRef.current = {
          cols: currentTerm.cols,
          rows: currentTerm.rows,
        };
        sessionResize(sessionId, currentTerm.cols, currentTerm.rows).catch(() => {});
      }
    };

    const resizeObserver = new ResizeObserver(() => {
      syncSize();
    });
    resizeObserver.observe(container);

    const dataDisposable = term.onData((data) => {
      sessionWrite(sessionId, data).catch(() => {});
    });
    term.attachCustomKeyEventHandler((event) => {
      const action = terminalClipboardAction(event, term.hasSelection());
      if (!action) return true;
      event.preventDefault();
      if (action === "copy") {
        void copySelection();
      } else {
        void pasteClipboard();
      }
      return false;
    });
    term.attachCustomWheelEventHandler((event) => {
      if (event.ctrlKey || event.metaKey) {
        return true;
      }
      const rows = terminalWheelRows(event.deltaY);
      const action = terminalWheelAction(
        host,
        term.buffer.active.type,
        trackingMode(term),
        event.deltaY > 0 ? 1 : -1,
        rows
      );
      if (action.kind === "delegate") {
        return true;
      }
      event.preventDefault();
      if (action.kind === "input") {
        void sessionWrite(sessionId, action.data);
      } else {
        showFeedback(action.hint);
      }
      return false;
    });
    const handlePointerDown = () => term.focus();
    container.addEventListener("pointerdown", handlePointerDown);

    // Output must be routed through the ref, never the effect-local terminal:
    // under StrictMode double-mounting and on unmount races, session-output
    // events and the attach snapshot can resolve after THIS mount's terminal
    // was disposed, and they must land on the live terminal instead. Writing
    // to a disposed terminal both loses the TUI's first frame (blank session)
    // and crashes the WebView on close.
    const applyOutput = (data: string) => {
      const current = terminalRef.current;
      if (!current) return;
      current.write(data, () => {
        if (terminalRef.current !== current) return;
        setMouseTracking(trackingMode(current));
      });
    };

    // Live events are buffered until the backstop snapshot has been written, so the
    // replayed pre-attach output (e.g. a TUI's first frame) always lands before any
    // live output and ordering is preserved.
    let attached = false;
    const pendingLive: string[] = [];
    const outputUnlisten = listen<{ id: string; data: string }>("session-output", (event) => {
      if (event.payload.id !== sessionId) {
        return;
      }
      if (attached) {
        applyOutput(event.payload.data);
      } else {
        pendingLive.push(event.payload.data);
      }
    });

    // Register the listener first, then attach: the backend replays everything it
    // buffered before this terminal existed and switches to live emission.
    outputUnlisten
      .then(() => sessionAttach(sessionId))
      .then((snapshot) => {
        if (!terminalRef.current) {
          return;
        }
        if (snapshot) {
          applyOutput(snapshot);
        }
        for (const chunk of pendingLive) {
          applyOutput(chunk);
        }
        pendingLive.length = 0;
        attached = true;
      })
      .catch(() => {});

    requestAnimationFrame(syncSize);

    return () => {
      resizeObserver.disconnect();
      dataDisposable.dispose();
      container.removeEventListener("pointerdown", handlePointerDown);
      outputUnlisten.then((fn) => fn());
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      lastSizeRef.current = null;
    };
  }, [copySelection, host, pasteClipboard, sessionId, showFeedback]);

  useEffect(() => {
    if (!active) {
      return;
    }

    requestAnimationFrame(() => {
      const term = terminalRef.current;
      const fitAddon = fitAddonRef.current;
      if (!term || !fitAddon) {
        return;
      }
      term.focus();
      fitAddon.fit();
      if (
        !lastSizeRef.current ||
        lastSizeRef.current.cols !== term.cols ||
        lastSizeRef.current.rows !== term.rows
      ) {
        lastSizeRef.current = { cols: term.cols, rows: term.rows };
        sessionResize(sessionId, term.cols, term.rows).catch(() => {});
      }
    });
  }, [active, sessionId]);

  const hint =
    mouseTracking !== "none"
      ? "Scroll with the mouse wheel. Hold Shift while dragging to select text."
      : "Scroll with the mouse wheel. Copy and paste use the standard terminal shortcuts.";

  return (
    <div
      style={{
        flex: 1,
        minHeight: 0,
        overflow: "hidden",
        background: "#120f18",
        boxSizing: "border-box",
        display: "flex",
        flexDirection: "column",
      }}
    >
      <div
        style={{
          minHeight: 34,
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "4px 10px",
          borderBottom: "1px solid rgba(57, 49, 70, 0.55)",
          background: "rgba(17, 14, 23, 0.96)",
          color: "var(--text-muted)",
          fontSize: 10.5,
          flexShrink: 0,
        }}
      >
        <span style={{ flex: 1, minWidth: 0 }}>{hint}</span>
        {host === "opencode" && (
          <span style={{ color: "var(--text-secondary)", flexShrink: 0 }}>
            Files: @workspace/
          </span>
        )}
        {feedback && (
          <span role="status" aria-live="polite" style={{ color: "var(--text-secondary)" }}>
            {feedback}
          </span>
        )}
        <button
          type="button"
          aria-pressed={showHistorySearch}
          onClick={() => setShowHistorySearch((prev) => !prev)}
          style={showHistorySearch ? toolbarButtonActiveStyle : toolbarButtonStyle}
        >
          Search logs
        </button>
      </div>
      
      <div style={{ display: "flex", flex: 1, minHeight: 0, overflow: "hidden" }}>
        {/* Keep padding outside the measured xterm element so FitAddon cannot overcount a row. */}
        <div style={{ flex: 1, minHeight: 0, padding: "8px 10px", boxSizing: "border-box" }}>
          <div ref={containerRef} style={{ width: "100%", height: "100%" }} />
        </div>
        {showHistorySearch && (
          <HistorySearchPanel
            sessionId={sessionId}
            onClose={() => setShowHistorySearch(false)}
          />
        )}
      </div>
    </div>
  );

}

const toolbarButtonStyle = {
  border: "1px solid #30283d",
  borderRadius: 7,
  background: "#191520",
  color: "var(--text-secondary)",
  fontSize: 10.5,
  fontWeight: 600,
  padding: "4px 8px",
  cursor: "pointer",
  flexShrink: 0,
} as const;

const toolbarButtonActiveStyle = {
  ...toolbarButtonStyle,
  border: "1px solid var(--accent-primary, #8f7ae5)",
  color: "var(--text-primary)",
  background: "#241d33",
} as const;
