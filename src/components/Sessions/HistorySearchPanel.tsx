import { useEffect, useState, useRef, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { sessionGetTranscript } from "../../lib/commands";

interface HistorySearchPanelProps {
  sessionId: string;
  onClose: () => void;
}

export function stripAnsi(str: string): string {
  return str.replace(/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, "");
}

function escapeRegExp(string: string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function HistorySearchPanel({ sessionId, onClose }: HistorySearchPanelProps) {
  const [rawTranscript, setRawTranscript] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [matchCase, setMatchCase] = useState(false);
  const [loading, setLoading] = useState(true);
  const [selectedLines, setSelectedLines] = useState<Record<number, boolean>>({});
  const [autoscroll, setAutoscroll] = useState(true);
  const [copiedFeedback, setCopiedFeedback] = useState(false);
  
  const scrollContainerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    let active = true;

    // Fetch complete transcript
    sessionGetTranscript(sessionId)
      .then((data) => {
        if (active) {
          setRawTranscript(data);
          setLoading(false);
        }
      })
      .catch((err) => {
        console.error("Failed to load transcript:", err);
        if (active) setLoading(false);
      });

    // Listen to real-time session outputs
    const unlistenPromise = listen<{ id: string; data: string }>("session-output", (event) => {
      if (event.payload.id === sessionId && active) {
        setRawTranscript((current) => current + event.payload.data);
      }
    });

    return () => {
      active = false;
      unlistenPromise.then((fn) => fn());
    };
  }, [sessionId]);

  const lines = useMemo(() => {
    if (!rawTranscript) return [];
    const cleanText = stripAnsi(rawTranscript);
    // Keep trailing line breaks
    return cleanText.split(/\r?\n/);
  }, [rawTranscript]);

  const filteredLines = useMemo(() => {
    if (!searchQuery.trim()) {
      return lines.map((text, idx) => ({ text, originalIndex: idx }));
    }
    const query = matchCase ? searchQuery : searchQuery.toLowerCase();
    return lines
      .map((text, idx) => ({ text, originalIndex: idx }))
      .filter((item) => {
        const lineText = matchCase ? item.text : item.text.toLowerCase();
        return lineText.includes(query);
      });
  }, [lines, searchQuery, matchCase]);

  // Autoscroll logic
  useEffect(() => {
    if (!autoscroll || !scrollContainerRef.current || searchQuery.trim() !== "") return;
    const container = scrollContainerRef.current;
    container.scrollTop = container.scrollHeight;
  }, [lines.length, autoscroll, searchQuery]);

  const handleScroll = () => {
    const container = scrollContainerRef.current;
    if (!container) return;
    const isAtBottom = container.scrollHeight - container.scrollTop - container.clientHeight < 20;
    setAutoscroll(isAtBottom);
  };

  const handleToggleSelect = (index: number) => {
    setSelectedLines((prev) => ({
      ...prev,
      [index]: !prev[index],
    }));
  };

  const selectedCount = useMemo(() => {
    return Object.values(selectedLines).filter(Boolean).length;
  }, [selectedLines]);

  const handleCopySelected = async () => {
    const selectedTexts = lines
      .map((text, idx) => (selectedLines[idx] ? text : null))
      .filter((t): t is string => t !== null);

    if (selectedTexts.length === 0) return;

    try {
      await navigator.clipboard.writeText(selectedTexts.join("\n"));
      setCopiedFeedback(true);
      setTimeout(() => setCopiedFeedback(false), 2000);
    } catch (err) {
      alert("Failed to copy selected text: " + err);
    }
  };

  const handleClearSelection = () => {
    setSelectedLines({});
  };

  const highlightMatch = (text: string, query: string) => {
    if (!query || !query.trim()) return <span>{text}</span>;
    try {
      const parts = text.split(new RegExp(`(${escapeRegExp(query)})`, matchCase ? "g" : "gi"));
      return (
        <span>
          {parts.map((part, i) =>
            part.toLowerCase() === query.toLowerCase() ? (
              <mark
                key={i}
                style={{
                  background: "rgba(124, 108, 246, 0.45)",
                  color: "#fff",
                  borderRadius: 2,
                  padding: "0 2px",
                }}
              >
                {part}
              </mark>
            ) : (
              part
            )
          )}
        </span>
      );
    } catch {
      return <span>{text}</span>;
    }
  };

  const visibleLimit = 1500;
  const limitedLines = filteredLines.slice(0, visibleLimit);

  return (
    <div
      style={{
        width: 440,
        borderLeft: "1px solid rgba(57, 49, 70, 0.8)",
        background: "rgba(13, 11, 19, 0.95)",
        display: "flex",
        flexDirection: "column",
        height: "100%",
        flexShrink: 0,
        boxSizing: "border-box",
      }}
    >
      {/* Header bar */}
      <div
        style={{
          padding: "12px 16px",
          borderBottom: "1px solid rgba(57, 49, 70, 0.6)",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
        }}
      >
        <span style={{ fontSize: 13, fontWeight: 700, color: "var(--text-primary)" }}>
          Session Transcript Search
        </span>
        <button
          onClick={onClose}
          style={{
            border: "none",
            background: "transparent",
            color: "var(--text-muted)",
            cursor: "pointer",
            padding: 4,
            display: "inline-flex",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
            close
          </i>
        </button>
      </div>

      {/* Control Actions & Search */}
      <div style={{ padding: "12px 16px", display: "flex", flexDirection: "column", gap: 8 }}>
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search logs..."
          style={{
            width: "100%",
            background: "var(--bg-primary)",
            border: "1px solid var(--border-primary)",
            borderRadius: 7,
            padding: "6px 10px",
            fontSize: 12.5,
            color: "var(--text-primary)",
            outline: "none",
            boxSizing: "border-box",
          }}
        />

        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <label style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 11.5, color: "var(--text-secondary)", cursor: "pointer" }}>
            <input
              type="checkbox"
              checked={matchCase}
              onChange={(e) => setMatchCase(e.target.checked)}
              style={{ cursor: "pointer" }}
            />
            Match case
          </label>

          <span style={{ fontSize: 11, color: "var(--text-tertiary)" }}>
            {filteredLines.length} / {lines.length} lines
          </span>
        </div>

        {/* Selection actions bar */}
        {selectedCount > 0 && (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              background: "rgba(124, 108, 246, 0.1)",
              border: "1px solid rgba(124, 108, 246, 0.25)",
              padding: "6px 10px",
              borderRadius: 6,
              marginTop: 4,
            }}
          >
            <span style={{ fontSize: 11, fontWeight: 600, color: "#bcaef6" }}>
              {selectedCount} lines selected
            </span>
            <div style={{ display: "flex", gap: 6 }}>
              <button
                onClick={handleCopySelected}
                style={{
                  border: "none",
                  borderRadius: 4,
                  background: "linear-gradient(135deg, #886ff7, #4d80f6)",
                  color: "#fff",
                  fontSize: 10,
                  fontWeight: 700,
                  padding: "4px 8px",
                  cursor: "pointer",
                }}
              >
                {copiedFeedback ? "Copied!" : "Copy Selected"}
              </button>
              <button
                onClick={handleClearSelection}
                style={{
                  border: "1px solid rgba(255,255,255,0.1)",
                  borderRadius: 4,
                  background: "transparent",
                  color: "var(--text-secondary)",
                  fontSize: 10,
                  fontWeight: 600,
                  padding: "4px 8px",
                  cursor: "pointer",
                }}
              >
                Clear
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Log lines area */}
      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        style={{
          flex: 1,
          overflowY: "auto",
          padding: "8px 16px 24px",
          fontFamily: "var(--font-mono)",
          fontSize: 11.5,
          lineHeight: 1.5,
          color: "var(--text-secondary)",
          boxSizing: "border-box",
        }}
      >
        {loading ? (
          <div style={{ textAlign: "center", padding: "40px 0", color: "var(--text-tertiary)" }}>
            Loading transcript...
          </div>
        ) : filteredLines.length === 0 ? (
          <div style={{ textAlign: "center", padding: "40px 0", color: "var(--text-tertiary)" }}>
            No matching lines found.
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column" }}>
            {limitedLines.map((line) => {
              const isSelected = !!selectedLines[line.originalIndex];
              return (
                <div
                  key={line.originalIndex}
                  onClick={() => handleToggleSelect(line.originalIndex)}
                  style={{
                    display: "flex",
                    alignItems: "flex-start",
                    padding: "2px 4px",
                    background: isSelected ? "rgba(124, 108, 246, 0.15)" : "transparent",
                    borderRadius: 4,
                    cursor: "pointer",
                    userSelect: "text",
                    borderLeft: isSelected ? "2px solid #8e7af8" : "2px solid transparent",
                  }}
                  onMouseEnter={(e) => {
                    if (!isSelected) e.currentTarget.style.background = "rgba(255,255,255,0.02)";
                  }}
                  onMouseLeave={(e) => {
                    if (!isSelected) e.currentTarget.style.background = "transparent";
                  }}
                >
                  {/* Line Number */}
                  <span
                    style={{
                      width: 32,
                      color: "var(--text-muted)",
                      fontSize: 10,
                      userSelect: "none",
                      marginRight: 8,
                      textAlign: "right",
                      flexShrink: 0,
                    }}
                  >
                    {line.originalIndex + 1}
                  </span>

                  {/* Text */}
                  <span
                    style={{
                      flex: 1,
                      whiteSpace: "pre-wrap",
                      wordBreak: "break-all",
                    }}
                  >
                    {highlightMatch(line.text, searchQuery)}
                  </span>
                </div>
              );
            })}

            {filteredLines.length > visibleLimit && (
              <div
                style={{
                  textAlign: "center",
                  padding: "12px 0",
                  fontSize: 11,
                  color: "var(--text-tertiary)",
                  borderTop: "1px solid rgba(255,255,255,0.05)",
                  marginTop: 10,
                }}
              >
                Showing first {visibleLimit} of {filteredLines.length} matching lines. Narrow down your search query.
              </div>
            )}
          </div>
        )}
      </div>

      {/* Autoscroll Floaty Badge */}
      {!autoscroll && searchQuery.trim() === "" && (
        <div
          onClick={() => {
            setAutoscroll(true);
            if (scrollContainerRef.current) {
              const container = scrollContainerRef.current;
              container.scrollTop = container.scrollHeight;
            }
          }}
          style={{
            position: "absolute",
            bottom: 16,
            right: 16,
            background: "#8e7af8",
            color: "#fff",
            padding: "6px 12px",
            borderRadius: 20,
            fontSize: 11,
            fontWeight: 700,
            cursor: "pointer",
            boxShadow: "0 4px 12px rgba(0,0,0,0.3)",
            display: "flex",
            alignItems: "center",
            gap: 4,
            zIndex: 10,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
            arrow_downward
          </i>
          New output available
        </div>
      )}
    </div>
  );
}
