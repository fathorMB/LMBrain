import { MarkdownRenderer } from "../../lib/markdown";
import { resolveWikilink } from "../../lib/wikilinks";
import type { WikiPage, WikiTree } from "../../types";

export interface WikiPageContentProps {
  currentPage: WikiPage | null;
  loading: boolean;
  resolvedTargets: Set<string>;
  tree: WikiTree | null;
  backlinks: string[];
  onNavigateToPage: (target: string) => void;
}

function InfoRow({
  label,
  value,
  mono,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
      }}
    >
      <span style={{ fontSize: "var(--text-sm)", color: "var(--text-tertiary)" }}>
        {label}
      </span>
      <span
        style={{
          fontSize: mono ? 12 : 12,
          fontFamily: mono ? "var(--font-mono)" : "inherit",
          color: "#cfc9d6",
        }}
      >
        {value}
      </span>
    </div>
  );
}

export function WikiPageContent({
  currentPage,
  loading,
  resolvedTargets,
  tree,
  backlinks,
  onNavigateToPage,
}: WikiPageContentProps) {
  return (
    <>
      {/* Center content */}
      <div style={{ flex: 1, minWidth: 0, minHeight: 0, overflowY: "auto" }}>
        {loading ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "var(--text-tertiary)",
            }}
          >
            Loading…
          </div>
        ) : currentPage ? (
          <div
            style={{
              maxWidth: "var(--page-reading)",
              margin: "0 auto",
              padding: "var(--page-top) var(--page-gutter-wide) var(--page-bottom)",
            }}
          >
            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                marginBottom: 18,
              }}
            >
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 7,
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-xs)",
                  color: "var(--text-tertiary)",
                }}
              >
                {currentPage.path.split("/").slice(-2).join("/")}
              </div>
            </div>

            <h1
              style={{
                fontSize: 30,
                fontWeight: 800,
                letterSpacing: "-.03em",
                margin: "0 0 6px",
              }}
            >
              {currentPage.name}
            </h1>

            <MarkdownRenderer
              content={currentPage.content_html}
              resolvedTargets={resolvedTargets}
              onWikilinkClick={onNavigateToPage}
            />
          </div>
        ) : (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "var(--text-tertiary)",
            }}
          >
            Select a file from the tree to view its content.
          </div>
        )}
      </div>

      {/* Right sidebar */}
      {currentPage && (
        <div
          style={{
            width: 268,
            flex: "none",
            minHeight: 0,
            borderLeft: "1px solid var(--border-primary)",
            overflowY: "auto",
            padding: "18px 16px",
            background: "#0e0c12",
          }}
        >
          <div
            style={{
              fontSize: "var(--text-xs)",
              letterSpacing: ".09em",
              textTransform: "uppercase",
              color: "var(--text-tertiary)",
              fontWeight: 600,
              marginBottom: 11,
            }}
          >
            Page info
          </div>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 9,
              marginBottom: 20,
            }}
          >
            {currentPage.updated && (
              <InfoRow label="Updated" value={currentPage.updated} />
            )}
            {currentPage.word_count && (
              <InfoRow
                label="Words"
                value={String(currentPage.word_count)}
                mono
              />
            )}
          </div>

          {/* Wikilinks (outgoing) */}
          {currentPage.wikilinks.length > 0 && (
            <>
              <div
                style={{
                  fontSize: "var(--text-xs)",
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "var(--text-tertiary)",
                  fontWeight: 600,
                  marginBottom: 10,
                }}
              >
                Wikilinks
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 5,
                  marginBottom: 20,
                }}
              >
                {currentPage.wikilinks.map((link, i) => {
                  const resolved = tree ? resolveWikilink(link, tree.root) : null;
                  return (
                    <button
                      key={i}
                      type="button"
                      disabled={!resolved}
                      onClick={() => resolved && onNavigateToPage(link)}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 7,
                        fontSize: "var(--text-sm)",
                        color: resolved ? "var(--accent-light)" : "var(--text-secondary)",
                        cursor: resolved ? "pointer" : "default",
                        border: "none",
                        background: "transparent",
                        padding: "2px 0",
                        textAlign: "left",
                        fontFamily: "inherit",
                      }}
                    >
                      <i
                        className="material-symbols-outlined"
                        style={{
                          fontSize: 14,
                          color: resolved ? "var(--accent-light)" : "var(--text-tertiary)",
                        }}
                      >
                        {resolved ? "link" : "link_off"}
                      </i>
                      {link}
                    </button>
                  );
                })}
              </div>
            </>
          )}

          {/* Backlinks */}
          {backlinks.length > 0 && (
            <>
              <div
                style={{
                  fontSize: "var(--text-xs)",
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "var(--text-tertiary)",
                  fontWeight: 600,
                  marginBottom: 10,
                }}
              >
                Backlinks
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 5,
                  marginBottom: 20,
                }}
              >
                {backlinks.map((bl, i) => (
                  <button
                    key={i}
                    type="button"
                    onClick={() => onNavigateToPage(bl)}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 7,
                      fontSize: "var(--text-sm)",
                      color: "var(--text-secondary)",
                      cursor: "pointer",
                      border: "none",
                      background: "transparent",
                      padding: "2px 0",
                      textAlign: "left",
                      fontFamily: "inherit",
                    }}
                  >
                    <i
                      className="material-symbols-outlined"
                      style={{ fontSize: 14, color: "var(--text-tertiary)" }}
                    >
                      link
                    </i>
                    {bl}
                  </button>
                ))}
              </div>
            </>
          )}
        </div>
      )}
    </>
  );
}
