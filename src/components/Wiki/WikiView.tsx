import { useEffect, useState, useMemo } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getWikiTree, getWikiPage, getWikilinkIndex } from "../../lib/commands";
import { resolveWikilink } from "../../lib/wikilinks";
import type { WikiNode, WikiPage } from "../../types";
import { TreeNode } from "./TreeNode";
import { WikiPageContent } from "./WikiPageContent";

export function WikiView() {
  const { state, setWikiTree, setWikiPage } = useWorkspace();
  const [currentPage, setCurrentPage] = useState<WikiPage | null>(null);
  const [loading, setLoading] = useState(false);
  const [wikilinkIndex, setWikilinkIndex] = useState<Record<string, string[]>>({});
  const [syncedWikiPage, setSyncedWikiPage] = useState<WikiPage | null>(null);

  if (state.wikiPage && state.wikiPage !== syncedWikiPage) {
    setSyncedWikiPage(state.wikiPage);
    setCurrentPage(state.wikiPage);
  }

  useEffect(() => {
    Promise.all([
      getWikiTree(),
      getWikilinkIndex(),
    ]).then(([tree, index]) => {
      setWikiTree(tree);
      setWikilinkIndex(index);
    }).catch(console.error);
  }, [setWikiTree]);

  const handleNodeClick = async (node: WikiNode) => {
    if (node.kind === "file") {
      setLoading(true);
      try {
        const fullPath = state.currentWorkspace
          ? `${state.currentWorkspace.path}/${node.path}`
          : node.path;
        const page = await getWikiPage(fullPath);
        setCurrentPage(page);
        setWikiPage(page);
      } catch (err) {
        console.error("Failed to load wiki page:", err);
      } finally {
        setLoading(false);
      }
    }
  };

  const tree = state.wikiTree;

  const resolvedTargets = useMemo(() => {
    const names = new Set<string>();
    function collectFileNames(node: WikiNode) {
      if (node.kind === "file") {
        const name = node.name.replace(/\.md$/i, "").toLowerCase();
        names.add(name);
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

  const backlinks = useMemo(() => {
    if (!currentPage) return [];
    const pageName = currentPage.path.split("/").pop()?.replace(/\.md$/i, "").toLowerCase() || "";
    const sources = wikilinkIndex[pageName.toLowerCase()] || [];
    const pathSources = wikilinkIndex[currentPage.path.toLowerCase()] || [];
    return [...new Set([...sources, ...pathSources])];
  }, [currentPage, wikilinkIndex]);

  const handleNavigateToPage = (target: string) => {
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
            setWikiPage(page);
          })
          .catch(console.error)
          .finally(() => setLoading(false));
      }
    }
  };

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
          padding: "var(--space-4) var(--space-3)",
          background: "#0e0c12",
        }}
      >
        <div
          style={{
            fontSize: "var(--text-2xs)",
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "var(--text-tertiary)",
            fontWeight: 600,
            marginBottom: 10,
            paddingLeft: 8,
          }}
        >
          Documentation
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 1 }}>
          {tree ? (
            <TreeNode
              node={tree.root}
              onSelect={handleNodeClick}
              depth={0}
            />
          ) : (
            <div
              style={{
                color: "var(--text-tertiary)",
                fontSize: "var(--text-sm)",
                padding: "8px",
              }}
            >
              Loading tree…
            </div>
          )}
        </div>
      </div>

      {/* Content & Right Sidebar */}
      <WikiPageContent
        currentPage={currentPage}
        loading={loading}
        resolvedTargets={resolvedTargets}
        tree={tree}
        backlinks={backlinks}
        onNavigateToPage={handleNavigateToPage}
      />
    </div>
  );
}
