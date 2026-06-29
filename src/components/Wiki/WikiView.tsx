import { useEffect, useState, useMemo } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getWikiTree, getWikiPage, getWikilinkIndex } from "../../lib/commands";
import { MarkdownRenderer } from "../../lib/markdown";
import { resolveWikilink } from "../../lib/wikilinks";
import type { WikiNode, WikiPage } from "../../types";

export function WikiView() {
  const { state, dispatch } = useWorkspace();
  const [currentPage, setCurrentPage] = useState<WikiPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [wikilinkIndex, setWikilinkIndex] = useState<Record<string, string[]>>({});
  const [syncedWikiPage, setSyncedWikiPage] = useState<WikiPage | null>(null);

  // Honor a page selected elsewhere (e.g. a [[wikilink]] clicked in the Pulse).
  // Adjusting state during render is the recommended pattern over a sync effect.
  if (state.wikiPage && state.wikiPage !== syncedWikiPage) {
    setSyncedWikiPage(state.wikiPage);
    setCurrentPage(state.wikiPage);
  }

  useEffect(() => {
    Promise.all([
      getWikiTree(),
      getWikilinkIndex(),
    ]).then(([tree, index]) => {
      dispatch({ type: "SET_WIKI_TREE", tree });
      setWikilinkIndex(index);
    }).catch(console.error);
  }, [dispatch]);

  const handleNodeClick = async (node: WikiNode) => {
    if (node.kind === "file") {
      setLoading(true);
      try {
        const fullPath = state.currentWorkspace
          ? `${state.currentWorkspace.path}/${node.path}`
          : node.path;
        const page = await getWikiPage(fullPath);
        setCurrentPage(page);
        dispatch({ type: "SET_WIKI_PAGE", page });
      } catch (err) {
        console.error("Failed to load wiki page:", err);
      } finally {
        setLoading(false);
      }
    }
  };

  const tree = state.wikiTree;

  // Compute resolved targets from the WikiTree file nodes (document existence)
  const resolvedTargets = useMemo(() => {
    const names = new Set<string>();
    function collectFileNames(node: WikiNode) {
      if (node.kind === "file") {
        // Add the file name (without .md extension, lowercased)
        const name = node.name.replace(/\.md$/i, "").toLowerCase();
        names.add(name);
        // Also add the full path for matching
        names.add(node.path.toLowerCase());
      }
      for (const child of node.children) {
        collectFileNames(child);
      }
    }
    if (state.wikiTree) {
      collectFileNames(state.wikiTree.root);
    }
    return names;
  }, [state.wikiTree]);

  // Compute backlinks for the current page from the real wikilink index
  const backlinks = useMemo(() => {
    if (!currentPage) return [];
    const pageName = currentPage.path.split("/").pop()?.replace(/\.md$/i, "").toLowerCase() || "";
    const sources = wikilinkIndex[pageName.toLowerCase()] || [];
    // Also check for the full path
    const pathSources = wikilinkIndex[currentPage.path.toLowerCase()] || [];
    const all = [...new Set([...sources, ...pathSources])];
    return all;
  }, [currentPage, wikilinkIndex]);

  return (
    <div style={{ display: "flex", height: "100%", minHeight: 0 }}>
      {/* Tree sidebar */}
      <div
        style={{
          width: 236,
          flex: "none",
          minHeight: 0,
          borderRight: "1px solid var(--border-primary)",
          overflowY: "auto",
          padding: "15px 11px",
          background: "#0e0c12",
        }}
      >
        <div
          style={{
            fontSize: 10,
            letterSpacing: ".1em",
            textTransform: "uppercase",
            color: "#56525b",
            fontWeight: 600,
            padding: "0 8px 9px",
          }}
        >
          .lmbrain
        </div>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 1,
            fontSize: 13,
          }}
        >
          {tree ? (
            <TreeNode node={tree.root} onSelect={handleNodeClick} depth={0} />
          ) : (
            <div
              style={{
                padding: "8px",
                color: "var(--text-tertiary)",
                fontSize: 12,
              }}
            >
              Loading…
            </div>
          )}
        </div>
      </div>

      {/* Content */}
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
            style={{ maxWidth: 760, margin: "0 auto", padding: "24px 40px 70px" }}
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
                  fontSize: 11.5,
                  color: "#6c6671",
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
              onWikilinkClick={(target) => {
                // Try to find and navigate to the target page
                if (tree) {
                  const resolved = resolveWikilink(target, tree.root);
                  if (resolved) {
                    const fullPath = state.currentWorkspace
                      ? `${state.currentWorkspace.path}/${resolved}`
                      : resolved;
                    setLoading(true);
                    getWikiPage(fullPath)
                      .then((page) => {
                        setCurrentPage(page);
                        dispatch({ type: "SET_WIKI_PAGE", page });
                      })
                      .catch(console.error)
                      .finally(() => setLoading(false));
                  }
                }
              }}
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
              fontSize: 10.5,
              letterSpacing: ".09em",
              textTransform: "uppercase",
              color: "#6c6671",
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
                  fontSize: 10.5,
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "#6c6671",
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
                  const resolved = tree
                    ? resolveWikilink(link, tree.root)
                    : null;
                  return (
                    <div
                      key={i}
                      style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 7,
                        fontSize: 12.5,
                        color: resolved ? "var(--accent-light)" : "#9a949f",
                        cursor: resolved ? "pointer" : "default",
                      }}
                    >
                      <i
                        className="material-symbols-outlined"
                        style={{
                          fontSize: 14,
                          color: resolved ? "var(--accent-light)" : "#6c6671",
                        }}
                      >
                        {resolved ? "link" : "link_off"}
                      </i>
                      {link}
                    </div>
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
                  fontSize: 10.5,
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "#6c6671",
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
                  <div
                    key={i}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 7,
                      fontSize: 12.5,
                      color: "#9a949f",
                      cursor: "pointer",
                    }}
                  >
                    <i
                      className="material-symbols-outlined"
                      style={{ fontSize: 14, color: "#6c6671" }}
                    >
                      link
                    </i>
                    {bl}
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      )}
    </div>
  );
}

function TreeNode({
  node,
  onSelect,
  depth,
}: {
  node: WikiNode;
  onSelect: (node: WikiNode) => void;
  depth: number;
}) {
  const isFile = node.kind === "file";
  const [expanded, setExpanded] = useState(depth === 0);
  const icon = isFile
    ? "article"
    : node.kind === "knowledge"
      ? "folder_open"
      : "folder";

  const handleToggle = (e: React.MouseEvent) => {
    if (isFile) {
      onSelect(node);
    } else {
      e.stopPropagation();
      setExpanded(!expanded);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isFile && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      setExpanded(!expanded);
    }
  };

  return (
    <div>
      <div
        role={isFile ? undefined : "button"}
        tabIndex={isFile ? undefined : 0}
        aria-expanded={isFile ? undefined : expanded}
        onClick={handleToggle}
        onKeyDown={handleKeyDown}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: `6px 8px 6px ${16 + depth * 20}px`,
          color: isFile ? "#9a949f" : "#b6b1bb",
          cursor: "pointer",
          borderRadius: 7,
          outline: "none",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.color = "var(--text-primary)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.color = isFile ? "#9a949f" : "#b6b1bb";
        }}
        onFocus={(e) => {
          if (!isFile) e.currentTarget.style.background = "#ffffff0c";
        }}
        onBlur={(e) => {
          if (!isFile) e.currentTarget.style.background = "transparent";
        }}
      >
        {/* Chevron for folders */}
        {!isFile && (
          <i
            className="material-symbols-outlined"
            style={{
              fontSize: 16,
              color: "#6c6671",
              userSelect: "none",
              marginRight: -4,
            }}
          >
            {expanded ? "expand_more" : "chevron_right"}
          </i>
        )}
        {/* Spacer for files to align them with folders having chevrons */}
        {isFile && <div style={{ width: 12 }} />}
        <i
          className="material-symbols-outlined"
          style={{
            fontSize: isFile ? 15 : 17,
            color: isFile ? "#6c6671" : "#8a858f",
          }}
        >
          {icon}
        </i>
        <span style={{ flex: 1 }}>{node.name}</span>
        {node.count !== null && node.count !== undefined && !isFile && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 10,
              color: "#56525b",
            }}
          >
            {node.count}
          </span>
        )}
      </div>
      {expanded && node.children.map((child, i) => (
        <TreeNode
          key={i}
          node={child}
          onSelect={onSelect}
          depth={depth + 1}
        />
      ))}
    </div>
  );
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
      <span style={{ fontSize: 12, color: "var(--text-tertiary)" }}>
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
