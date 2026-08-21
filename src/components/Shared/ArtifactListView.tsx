import { type ReactNode } from "react";
import { CardGrid, EmptyState, PageHeader, PageShell } from "./PageLayout";

export interface ArtifactListViewProps<T> {
  title: string;
  description?: string;
  headerActions?: ReactNode;
  summary?: ReactNode;
  filterBar?: ReactNode;
  items: T[];
  totalCount: number;
  loading?: boolean;
  loadingMessage?: string;
  error?: string | null;
  emptyAllMessage?: string;
  emptyFilteredMessage?: string;
  renderItem?: (item: T, index: number) => ReactNode;
  children?: ReactNode;
}

export function ArtifactListView<T>({
  title,
  description,
  headerActions,
  summary,
  filterBar,
  items,
  totalCount,
  loading = false,
  loadingMessage = "Loading…",
  error = null,
  emptyAllMessage = "No items recorded yet.",
  emptyFilteredMessage = "No items match these filters.",
  renderItem,
  children,
}: ArtifactListViewProps<T>) {
  return (
    <PageShell archetype="dense">
      <PageHeader
        title={title}
        description={description}
        actions={headerActions}
      />

      {error && (
        <div
          role="alert"
          style={{
            border: "1px solid rgba(224,88,74,.25)",
            background: "rgba(224,88,74,.08)",
            color: "#f87171",
            borderRadius: 8,
            padding: "10px 14px",
            marginBottom: 16,
            fontSize: "var(--text-sm)",
          }}
        >
          {error}
        </div>
      )}

      {loading && (
        <p
          role="status"
          style={{
            color: "var(--text-tertiary)",
            fontSize: "var(--text-sm)",
            marginBottom: 14,
          }}
        >
          {loadingMessage}
        </p>
      )}

      {summary}

      {filterBar}

      {totalCount === 0 && !loading && (
        <EmptyState>{emptyAllMessage}</EmptyState>
      )}

      {totalCount > 0 && items.length === 0 && !loading && (
        <EmptyState>{emptyFilteredMessage}</EmptyState>
      )}

      {items.length > 0 && (
        renderItem ? (
          <CardGrid>
            {items.map((item, index) => renderItem(item, index))}
          </CardGrid>
        ) : (
          children
        )
      )}
    </PageShell>
  );
}
