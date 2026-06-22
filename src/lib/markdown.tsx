import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeSanitize from "rehype-sanitize";
import type { Components } from "react-markdown";
import { preprocessWikilinksToMarkdown } from "./remark-wikilinks";

interface MarkdownRendererProps {
  content: string;
  onWikilinkClick?: (target: string) => void;
  /** Set of known/resolved wikilink targets (lowercase). */
  resolvedTargets?: Set<string>;
  className?: string;
}

/**
 * Custom link handler that intercepts `#wikilink:` fragment URLs
 * and renders them as local navigation controls.
 *
 * Fragment-only URLs (starting with `#`) are always allowed by the
 * default rehype-sanitize schema, so no schema extension is needed.
 */
function WikilinkHandler({
  href,
  children,
  onWikilinkClick,
  resolvedTargets,
}: {
  href?: string;
  children?: React.ReactNode;
  onWikilinkClick?: (target: string) => void;
  resolvedTargets?: Set<string>;
}) {
  if (href?.startsWith("#wikilink:")) {
    const target = decodeURIComponent(href.slice(10));
    const isResolved = resolvedTargets?.has(target.toLowerCase());

    if (isResolved) {
      // Resolved wikilink: clickable navigation control
      return (
        <span
          onClick={(e) => {
            e.preventDefault();
            onWikilinkClick?.(target);
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              onWikilinkClick?.(target);
            }
          }}
          tabIndex={0}
          role="button"
          aria-label={`Navigate to ${target}`}
          style={{
            color: "var(--accent-light)",
            borderBottom: "1px dashed rgba(165,150,245,.5)",
            cursor: "pointer",
            fontFamily: "var(--font-mono)",
            fontSize: "0.95em",
          }}
          title={`Navigate to ${target}`}
        >
          {children || target}
        </span>
      );
    }

    // Unresolved wikilink: muted, non-interactive text
    return (
      <span
        style={{
          color: "#6c6671",
          fontFamily: "var(--font-mono)",
          fontSize: "0.95em",
          cursor: "default",
        }}
        title={`Unresolved link: ${target}`}
      >
        {children || target}
      </span>
    );
  }
  // Regular external link
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      style={{ color: "var(--accent-light)", textDecoration: "underline" }}
    >
      {children}
    </a>
  );
}

const defaultComponents: Components = {
  a: ({ href, children }) => (
    <WikilinkHandler href={href} onWikilinkClick={undefined}>
      {children}
    </WikilinkHandler>
  ),
  code: ({ className, children, ...props }) => {
    const isInline = !className;
    if (isInline) {
      return (
        <code
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: "0.9em",
            background: "rgba(124,108,246,.1)",
            padding: "1px 5px",
            borderRadius: 4,
            color: "#bcaef6",
          }}
          {...props}
        >
          {children}
        </code>
      );
    }
    return (
      <pre
        style={{
          background: "#0e0c12",
          border: "1px solid #221f29",
          borderRadius: 10,
          padding: "14px 16px",
          overflowX: "auto",
          fontFamily: "var(--font-mono)",
          fontSize: 12.5,
          lineHeight: 1.85,
          color: "#b6b1bb",
        }}
      >
        <code {...props}>{children}</code>
      </pre>
    );
  },
  table: ({ children }) => (
    <div style={{ overflowX: "auto", marginBottom: 16 }}>
      <table
        style={{
          borderCollapse: "collapse",
          width: "100%",
          fontSize: 13,
          color: "#c2bdc8",
        }}
      >
        {children}
      </table>
    </div>
  ),
  th: ({ children }) => (
    <th
      style={{
        borderBottom: "2px solid #2a2731",
        padding: "8px 12px",
        textAlign: "left",
        fontWeight: 600,
        color: "var(--text-primary)",
      }}
    >
      {children}
    </th>
  ),
  td: ({ children }) => (
    <td
      style={{
        borderBottom: "1px solid #221f29",
        padding: "8px 12px",
      }}
    >
      {children}
    </td>
  ),
  blockquote: ({ children }) => (
    <div
      style={{
        display: "flex",
        gap: 12,
        background: "rgba(124,108,246,.08)",
        border: "1px solid rgba(124,108,246,.22)",
        borderLeft: "3px solid var(--accent)",
        borderRadius: 10,
        padding: "13px 15px",
        marginBottom: 22,
        color: "#c2bdc8",
        fontSize: 13.5,
        lineHeight: 1.6,
      }}
    >
      {children}
    </div>
  ),
  ul: ({ children }) => (
    <ul style={{ paddingLeft: 18, lineHeight: 1.8, color: "#c2bdc8" }}>
      {children}
    </ul>
  ),
  ol: ({ children }) => (
    <ol style={{ paddingLeft: 18, lineHeight: 1.8, color: "#c2bdc8" }}>
      {children}
    </ol>
  ),
  h1: ({ children }) => (
    <h1
      style={{
        fontSize: 30,
        fontWeight: 800,
        letterSpacing: "-.03em",
        margin: "0 0 6px",
      }}
    >
      {children}
    </h1>
  ),
  h2: ({ children }) => (
    <h2
      style={{
        fontSize: 19,
        fontWeight: 700,
        letterSpacing: "-.01em",
        margin: "24px 0 11px",
      }}
    >
      {children}
    </h2>
  ),
  h3: ({ children }) => (
    <h3
      style={{
        fontSize: 16,
        fontWeight: 600,
        margin: "20px 0 8px",
      }}
    >
      {children}
    </h3>
  ),
  p: ({ children }) => (
    <p
      style={{
        fontSize: 14.5,
        lineHeight: 1.72,
        color: "#c2bdc8",
        margin: "0 0 16px",
      }}
    >
      {children}
    </p>
  ),
  hr: () => (
    <div
      style={{
        height: 1,
        background: "#221f29",
        margin: "20px 0",
      }}
    />
  ),
};

export function MarkdownRenderer({
  content,
  onWikilinkClick,
  resolvedTargets,
  className,
}: MarkdownRendererProps) {
  // Preprocess [[wikilinks]] to standard markdown links with #wikilink: fragment
  const processed = preprocessWikilinksToMarkdown(content);

  // Create components with the wikilink click handler and resolution context injected
  const components: Partial<Components> = {
    ...defaultComponents,
    a: ({ href, children }) => (
      <WikilinkHandler
        href={href}
        onWikilinkClick={onWikilinkClick}
        resolvedTargets={resolvedTargets}
      >
        {children}
      </WikilinkHandler>
    ),
  };

  return (
    <div className={className}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeSanitize]}
        components={components}
      >
        {processed}
      </ReactMarkdown>
    </div>
  );
}
