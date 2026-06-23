import { useState, useRef, useEffect } from "react";
import { useWorkspace } from "../hooks/useWorkspace";
import { searchContent } from "../lib/commands";
import type { SearchResult } from "../lib/commands";

export function CommandPalette() {
  const { state, navigateTo, closeCmdk, goToPicker } = useWorkspace();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    if (query.length < 2) {
      return;
    }
    let cancelled = false;
    const timer = setTimeout(async () => {
      setSearching(true);
      try {
        const res = await searchContent(query);
        if (!cancelled) {
          setResults(res);
        }
      } catch {
        if (!cancelled) setResults([]);
      } finally {
        if (!cancelled) setSearching(false);
      }
    }, 300);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [query]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      closeCmdk();
    }
  };

  return (
    <>
      {/* Scrim */}
      <div
        onClick={closeCmdk}
        style={{
          position: "fixed",
          inset: 0,
          background: "rgba(6,5,8,.6)",
          zIndex: 50,
          display: "flex",
          alignItems: "flex-start",
          justifyContent: "center",
          paddingTop: "14vh",
        }}
      />
      {/* Palette */}
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          position: "fixed",
          top: "14vh",
          left: "50%",
          transform: "translateX(-50%)",
          width: 580,
          maxWidth: "92vw",
          background: "#15131a",
          border: "1px solid #2e2a37",
          borderRadius: 14,
          overflow: "hidden",
          boxShadow: "0 40px 90px -30px rgba(0,0,0,.85)",
          zIndex: 51,
        }}
      >
        {/* Search input */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 11,
            padding: "14px 17px",
            borderBottom: "1px solid #232029",
          }}
        >
          <i
            className="material-symbols-outlined"
            style={{ fontSize: 20, color: "#6c6671" }}
          >
            search
          </i>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search specs, files…"
            style={{
              flex: 1,
              fontSize: 14.5,
              color: "var(--text-primary)",
              background: "transparent",
              border: "none",
              outline: "none",
              fontFamily: "inherit",
            }}
          />
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 10,
              color: "#6c6671",
              border: "1px solid #2b2833",
              borderRadius: 5,
              padding: "2px 6px",
            }}
          >
            ESC
          </span>
        </div>

        {/* Navigation items (shown when no search query) */}
        {query.length < 2 && (
          <div style={{ padding: 8 }}>
            <div
              style={{
                fontSize: 10,
                letterSpacing: ".09em",
                textTransform: "uppercase",
                color: "#56525b",
                fontWeight: 600,
                padding: "8px 10px 6px",
              }}
            >
              Navigate
            </div>
            {[
              { icon: "monitoring", label: "Go to Project Pulse", shortcut: "G P", view: "pulse" as const },
              { icon: "view_kanban", label: "Open Board", shortcut: "G T", view: "taskboard" as const },
              { icon: "menu_book", label: "Browse Wiki", shortcut: "G W", view: "wiki" as const },
              { icon: "rate_review", label: "View Reviews", shortcut: "G R", view: "reviews" as const },
              { icon: "account_balance", label: "View Decisions", shortcut: "G D", view: "decisions" as const },
              { icon: "smart_toy", label: "View Agents & MCP", shortcut: "G A", view: "agents" as const },
            ].map((item) => (
              <div
                key={item.view}
                onClick={() => navigateTo(item.view)}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 12,
                  padding: "9px 11px",
                  borderRadius: 9,
                  cursor: "pointer",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = "#221e2b";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = "transparent";
                }}
              >
                <i
                  className="material-symbols-outlined"
                  style={{ fontSize: 18, color: "var(--accent-light)" }}
                >
                  {item.icon}
                </i>
                <span
                  style={{
                    flex: 1,
                    fontSize: 13.5,
                    color: "var(--text-primary)",
                  }}
                >
                  {item.label}
                </span>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: 10.5,
                    color: "#6c6671",
                  }}
                >
                  {item.shortcut}
                </span>
              </div>
            ))}

            <div
              style={{
                fontSize: 10,
                letterSpacing: ".09em",
                textTransform: "uppercase",
                color: "#56525b",
                fontWeight: 600,
                padding: "12px 10px 6px",
              }}
            >
              Actions
            </div>
            <div
              onClick={() => {
                const readySpec = state.specs.find((s) => s.status === "ready");
                if (readySpec) {
                  navigator.clipboard.writeText(
                    `Implement ${readySpec.id}: ${readySpec.title}`
                  );
                }
                closeCmdk();
              }}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                padding: "9px 11px",
                borderRadius: 9,
                cursor: "pointer",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "#221e2b";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "transparent";
              }}
            >
              <i
                className="material-symbols-outlined"
                style={{ fontSize: 18, color: "var(--text-tertiary)" }}
              >
                content_copy
              </i>
              <span
                style={{
                  flex: 1,
                  fontSize: 13.5,
                  color: "var(--text-primary)",
                }}
              >
                Copy handoff prompt
              </span>
            </div>
            <div
              onClick={() => goToPicker()}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 12,
                padding: "9px 11px",
                borderRadius: 9,
                cursor: "pointer",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = "#221e2b";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = "transparent";
              }}
            >
              <i
                className="material-symbols-outlined"
                style={{ fontSize: 18, color: "var(--text-tertiary)" }}
              >
                swap_horiz
              </i>
              <span
                style={{
                  flex: 1,
                  fontSize: 13.5,
                  color: "var(--text-primary)",
                }}
              >
                Switch repository…
              </span>
            </div>
          </div>
        )}

        {/* Search results */}
        {query.length >= 2 && (
          <div style={{ padding: 8, maxHeight: 300, overflowY: "auto" }}>
            {searching && (
              <div
                style={{
                  padding: "12px",
                  textAlign: "center",
                  color: "var(--text-tertiary)",
                  fontSize: 13,
                }}
              >
                Searching…
              </div>
            )}
            {!searching && results.length === 0 && (
              <div
                style={{
                  padding: "12px",
                  textAlign: "center",
                  color: "var(--text-tertiary)",
                  fontSize: 13,
                }}
              >
                No results found.
              </div>
            )}
            {results.map((r, i) => (
              <div
                key={i}
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 4,
                  padding: "9px 11px",
                  borderRadius: 9,
                  cursor: "pointer",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.background = "#221e2b";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.background = "transparent";
                }}
              >
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                  }}
                >
                  <i
                    className="material-symbols-outlined"
                    style={{ fontSize: 16, color: "var(--accent-light)" }}
                  >
                    description
                  </i>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: 12,
                      color: "#bcaef6",
                    }}
                  >
                    {r.path}
                  </span>
                </div>
                <div
                  style={{
                    fontSize: 12,
                    color: "var(--text-tertiary)",
                    lineHeight: 1.4,
                    paddingLeft: 24,
                  }}
                >
                  {r.snippet}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
