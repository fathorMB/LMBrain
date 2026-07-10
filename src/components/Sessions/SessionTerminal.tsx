import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { sessionAttach, sessionResize, sessionWrite } from "../../lib/commands";
import { terminalClipboardAction } from "../../lib/terminalClipboard";

interface SessionTerminalProps {
  sessionId: string;
  active: boolean;
}

export function SessionTerminal({ sessionId, active }: SessionTerminalProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const lastSizeRef = useRef<{ cols: number; rows: number } | null>(null);
  const feedbackTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [clipboardFeedback, setClipboardFeedback] = useState<string | null>(null);

  const showClipboardFeedback = useCallback((message: string) => {
    setClipboardFeedback(message);
    if (feedbackTimeoutRef.current) clearTimeout(feedbackTimeoutRef.current);
    feedbackTimeoutRef.current = setTimeout(() => {
      setClipboardFeedback(null);
      feedbackTimeoutRef.current = null;
    }, 2400);
  }, []);

  const copySelection = useCallback(async () => {
    const selection = terminalRef.current?.getSelection() ?? "";
    if (!selection) {
      showClipboardFeedback("Select terminal text before copying.");
      return;
    }
    if (!navigator.clipboard) {
      showClipboardFeedback("Clipboard access is unavailable.");
      return;
    }
    try {
      await navigator.clipboard.writeText(selection);
      showClipboardFeedback("Selection copied.");
    } catch {
      showClipboardFeedback("Could not copy the terminal selection.");
    }
  }, [showClipboardFeedback]);

  const pasteClipboard = useCallback(async () => {
    const term = terminalRef.current;
    if (!term || !navigator.clipboard) {
      showClipboardFeedback("Clipboard access is unavailable.");
      return;
    }
    try {
      const text = await navigator.clipboard.readText();
      if (!text) {
        showClipboardFeedback("Clipboard is empty.");
        return;
      }
      term.paste(text);
      term.focus();
      showClipboardFeedback("Clipboard pasted.");
    } catch {
      showClipboardFeedback("Could not read from the clipboard.");
    }
  }, [showClipboardFeedback]);

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
    const handleWheel = (event: WheelEvent) => {
      if (event.ctrlKey || event.metaKey) {
        return;
      }
      const rows = Math.max(1, Math.ceil(Math.abs(event.deltaY) / 36));
      term.scrollLines(event.deltaY > 0 ? rows : -rows);
      event.preventDefault();
    };
    container.addEventListener("wheel", handleWheel, { passive: false });
    const handlePointerDown = () => term.focus();
    container.addEventListener("pointerdown", handlePointerDown);

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
        terminalRef.current?.write(event.payload.data);
      } else {
        pendingLive.push(event.payload.data);
      }
    });

    // Register the listener first, then attach: the backend replays everything it
    // buffered before this terminal existed and switches to live emission.
    outputUnlisten
      .then(() => sessionAttach(sessionId))
      .then((snapshot) => {
        const term = terminalRef.current;
        if (!term) {
          return;
        }
        if (snapshot) {
          term.write(snapshot);
        }
        for (const chunk of pendingLive) {
          term.write(chunk);
        }
        pendingLive.length = 0;
        attached = true;
      })
      .catch(() => {});

    requestAnimationFrame(syncSize);

    return () => {
      resizeObserver.disconnect();
      dataDisposable.dispose();
      container.removeEventListener("wheel", handleWheel);
      container.removeEventListener("pointerdown", handlePointerDown);
      outputUnlisten.then((fn) => fn());
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      lastSizeRef.current = null;
    };
  }, [copySelection, pasteClipboard, sessionId]);

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
        <span style={{ flex: 1, minWidth: 0 }}>
          Copy: select + Ctrl+C, Ctrl+Shift+C, or Cmd+C · Paste: Ctrl+Shift+V or Cmd+V
        </span>
        {clipboardFeedback && (
          <span role="status" aria-live="polite" style={{ color: "var(--text-secondary)" }}>
            {clipboardFeedback}
          </span>
        )}
        <button type="button" onClick={() => void copySelection()} style={clipboardButtonStyle}>
          Copy
        </button>
        <button type="button" onClick={() => void pasteClipboard()} style={clipboardButtonStyle}>
          Paste
        </button>
      </div>
      {/* Keep padding outside the measured xterm element so FitAddon cannot overcount a row. */}
      <div style={{ flex: 1, minHeight: 0, padding: "8px 10px", boxSizing: "border-box" }}>
        <div ref={containerRef} style={{ width: "100%", height: "100%" }} />
      </div>
    </div>
  );
}

const clipboardButtonStyle = {
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
