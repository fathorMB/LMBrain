import { useState } from "react";
import type { WikiNode } from "../../types";

export interface TreeNodeProps {
  node: WikiNode;
  onSelect: (node: WikiNode) => void;
  depth: number;
}

export function TreeNode({ node, onSelect, depth }: TreeNodeProps) {
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
    if (isFile && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      onSelect(node);
    } else if (!isFile && (e.key === "Enter" || e.key === " ")) {
      e.preventDefault();
      setExpanded(!expanded);
    }
  };

  return (
    <div>
      <div
        role="button"
        tabIndex={0}
        aria-expanded={isFile ? undefined : expanded}
        onClick={handleToggle}
        onKeyDown={handleKeyDown}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: `6px 8px 6px ${16 + depth * 20}px`,
          color: isFile ? "var(--text-secondary)" : "#b6b1bb",
          cursor: "pointer",
          borderRadius: 7,
          outline: "none",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.color = "var(--text-primary)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.color = isFile ? "var(--text-secondary)" : "#b6b1bb";
        }}
        onFocus={(e) => {
          e.currentTarget.style.background = "#ffffff0c";
        }}
        onBlur={(e) => {
          e.currentTarget.style.background = "transparent";
        }}
      >
        {/* Chevron for folders */}
        {!isFile && (
          <i
            className="material-symbols-outlined"
            style={{
              fontSize: 16,
              color: "var(--text-tertiary)",
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
            color: isFile ? "var(--text-tertiary)" : "#8a858f",
          }}
        >
          {icon}
        </i>
        <span style={{ flex: 1 }}>{node.name}</span>
        {node.count !== null && node.count !== undefined && !isFile && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-2xs)",
              color: "var(--text-muted)",
            }}
          >
            {node.count}
          </span>
        )}
      </div>
      {expanded &&
        node.children.map((child, i) => (
          <TreeNode key={i} node={child} onSelect={onSelect} depth={depth + 1} />
        ))}
    </div>
  );
}
