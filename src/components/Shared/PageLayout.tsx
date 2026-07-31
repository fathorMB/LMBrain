import type { CSSProperties, ReactNode } from "react";
import "./layout.css";

/**
 * Page archetypes (ISSUE-50-PLAN.md §3.2):
 *
 *   reading — prose and single-artifact detail, capped at a readable measure.
 *   dense   — the default workspace surface: fluid up to the width cap.
 *   full    — full-bleed shells whose panes own their own scrolling.
 */
export type PageArchetype = "reading" | "dense" | "full";

interface PageShellProps {
  archetype: PageArchetype;
  children: ReactNode;
  /** Escape hatch for pages that need their own inner layout. */
  className?: string;
  style?: CSSProperties;
}

export function PageShell({ archetype, children, className, style }: PageShellProps) {
  const shellClassName = ["lm-page", `lm-page--${archetype}`, className].filter(Boolean).join(" ");
  return (
    <div className={shellClassName} data-archetype={archetype} style={style}>
      {archetype === "full" ? children : <div className="lm-page__inner">{children}</div>}
    </div>
  );
}

interface PageHeaderProps {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
}

export function PageHeader({ title, description, actions }: PageHeaderProps) {
  return (
    <header className="lm-page-header">
      <div>
        <h1 className="lm-page-header__title">{title}</h1>
        {description !== undefined && description !== null && (
          <p className="lm-page-header__description">{description}</p>
        )}
      </div>
      {actions ? <div className="lm-page-header__actions">{actions}</div> : null}
    </header>
  );
}

interface PageSectionProps {
  title?: ReactNode;
  description?: ReactNode;
  children: ReactNode;
}

export function PageSection({ title, description, children }: PageSectionProps) {
  return (
    <section className="lm-page-section">
      {title !== undefined && title !== null && <h2 className="lm-page-section__title">{title}</h2>}
      {description !== undefined && description !== null && (
        <p className="lm-page-section__description">{description}</p>
      )}
      {children}
    </section>
  );
}

export function Toolbar({ children }: { children: ReactNode }) {
  return <div className="lm-toolbar">{children}</div>;
}

interface CardGridProps {
  children: ReactNode;
  /** Narrower cards yield more columns; 360 is the approved default (§7). */
  minColumnWidth?: number;
}

export function CardGrid({ children, minColumnWidth = 360 }: CardGridProps) {
  return (
    <div
      className="lm-card-grid"
      data-min-column-width={minColumnWidth}
      style={{ "--lm-card-min": `${minColumnWidth}px` } as CSSProperties}
    >
      {children}
    </div>
  );
}

export function EmptyState({ children }: { children: ReactNode }) {
  return <div className="lm-empty-state">{children}</div>;
}
