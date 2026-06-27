import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import { sessionAttach, sessionResize, sessionWrite } from "../../lib/commands";

interface SessionTerminalProps {
  sessionId: string;
  active: boolean;
}

export function SessionTerminal({ sessionId, active }: SessionTerminalProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const terminalRef = useRef<Terminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const lastSizeRef = useRef<{ cols: number; rows: number } | null>(null);

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
      outputUnlisten.then((fn) => fn());
      term.dispose();
      terminalRef.current = null;
      fitAddonRef.current = null;
      lastSizeRef.current = null;
    };
  }, [sessionId]);

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
    // No padding on the measured container: with a global `box-sizing: border-box`,
    // padding here inflates FitAddon's height measurement and renders one extra row
    // that overflows past the window. Padding lives on an inner wrapper instead.
    <div
      style={{
        flex: 1,
        minHeight: 0,
        overflow: "hidden",
        background: "#120f18",
        padding: "8px 10px",
        boxSizing: "border-box",
      }}
    >
      <div ref={containerRef} style={{ width: "100%", height: "100%" }} />
    </div>
  );
}
