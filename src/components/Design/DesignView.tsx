import { useEffect, useMemo, useState } from "react";
import { getDesignMockups, readDesignMockupPreviewHtml } from "../../lib/commands";
import type { DesignMockup } from "../../types";

export function DesignView() {
  const [mockups, setMockups] = useState<DesignMockup[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [previewError, setPreviewError] = useState<string | null>(null);
  const [previewHtml, setPreviewHtml] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    getDesignMockups()
      .then((items) => {
        if (!alive) return;
        setMockups(items);
        setSelectedId(items[0]?.id ?? null);
        setError(null);
      })
      .catch(() => {
        if (!alive) return;
        setError("Unable to load design mockups.");
      })
      .finally(() => {
        if (alive) setLoading(false);
      });
    return () => {
      alive = false;
    };
  }, []);

  const selected = useMemo(
    () => mockups.find((mockup) => mockup.id === selectedId) ?? null,
    [mockups, selectedId]
  );

  useEffect(() => {
    if (!selected) {
      return;
    }

    let alive = true;
    readDesignMockupPreviewHtml(selected.entry_path)
      .then((html) => {
        if (!alive) return;
        setPreviewHtml(html.content);
      })
      .catch(() => {
        if (!alive) return;
        setPreviewHtml(null);
        setPreviewError("Preview unavailable for this design mockup.");
      });

    return () => {
      alive = false;
    };
  }, [selected]);

  const handleSelectMockup = (id: string) => {
    setSelectedId(id);
    setPreviewHtml(null);
    setPreviewError(null);
  };

  const previewLoading = !!selected && !previewHtml && !previewError;

  if (loading) {
    return <CenteredState icon="hourglass_top" title="Loading designs" />;
  }

  if (error) {
    return <CenteredState icon="error" title={error} />;
  }

  if (mockups.length === 0) {
    return (
      <CenteredState
        icon="design_services"
        title="No design mockups"
        description="Add self-contained HTML mockups under .lmbrain/design to review them here."
      />
    );
  }

  return (
    <div style={{ display: "flex", height: "100%", minHeight: 0 }}>
      <div
        style={{
          width: 328,
          flex: "none",
          minHeight: 0,
          overflowY: "auto",
          borderRight: "1px solid var(--border-primary)",
          background: "#0e0c12",
          padding: "18px 14px",
        }}
      >
        <div
          style={{
            fontSize: 10.5,
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "#6c6671",
            fontWeight: 600,
            margin: "0 0 10px 4px",
          }}
        >
          Design Mockups
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
          {mockups.map((mockup) => (
            <button
              key={mockup.id}
              onClick={() => handleSelectMockup(mockup.id)}
              style={{
                textAlign: "left",
                border: `1px solid ${
                  selectedId === mockup.id ? "rgba(124,108,246,.55)" : "var(--border-secondary)"
                }`,
                background: selectedId === mockup.id ? "#17131f" : "var(--bg-tertiary)",
                borderRadius: 8,
                padding: "12px 13px",
                color: "var(--text-primary)",
                cursor: "pointer",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: 9 }}>
                <i
                  className="material-symbols-outlined"
                  style={{ fontSize: 18, color: "var(--accent-light)" }}
                >
                  {mockup.kind === "package" ? "folder_special" : "html"}
                </i>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ fontSize: 13.5, fontWeight: 700 }}>
                    {mockup.manifest_title || mockup.name}
                  </div>
                  <div
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: 10.5,
                      color: "var(--text-tertiary)",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {mockup.entry_path}
                  </div>
                </div>
              </div>
              {mockup.summary && (
                <div
                  style={{
                    fontSize: 12,
                    color: "var(--text-tertiary)",
                    lineHeight: 1.45,
                    marginTop: 8,
                  }}
                >
                  {mockup.summary}
                </div>
              )}
            </button>
          ))}
        </div>
      </div>

      <div style={{ flex: 1, minWidth: 0, minHeight: 0, display: "flex", flexDirection: "column" }}>
        {selected && (
          <div
            style={{
              borderBottom: "1px solid var(--border-primary)",
              padding: "16px 22px",
              display: "flex",
              alignItems: "center",
              gap: 18,
              background: "#0c0b0f",
            }}
          >
            <div style={{ flex: 1, minWidth: 0 }}>
              <h1 style={{ fontSize: 21, fontWeight: 800, margin: "0 0 4px" }}>
                {selected.manifest_title || selected.name}
              </h1>
              <div
                style={{
                  display: "flex",
                  gap: 12,
                  color: "var(--text-tertiary)",
                  fontFamily: "var(--font-mono)",
                  fontSize: 11,
                  flexWrap: "wrap",
                }}
              >
                <span>{selected.kind === "package" ? "package" : "html"}</span>
                <span>{formatBytes(selected.size)}</span>
                {selected.modified && <span>{selected.modified}</span>}
                {selected.has_manifest && <span>manifest</span>}
                {selected.has_readme && <span>readme</span>}
              </div>
            </div>
            <button
              onClick={() => navigator.clipboard?.writeText(selected.entry_path)}
              style={{
                border: "1px solid var(--border-secondary)",
                background: "var(--bg-tertiary)",
                color: "var(--text-secondary)",
                borderRadius: 8,
                padding: "8px 10px",
                cursor: "pointer",
                display: "flex",
                alignItems: "center",
                gap: 7,
                fontSize: 12,
              }}
              title="Copy entry path"
            >
              <i className="material-symbols-outlined" style={{ fontSize: 16 }}>
                content_copy
              </i>
              Copy path
            </button>
          </div>
        )}

        <div style={{ flex: 1, minHeight: 0, padding: 18, background: "#09080b" }}>
          <div
            style={{
              height: "100%",
              minHeight: 0,
              border: "1px solid var(--border-secondary)",
              borderRadius: 8,
              overflow: "hidden",
              background: "#fff",
            }}
          >
            {previewLoading ? (
              <CenteredState icon="hourglass_top" title="Loading preview" light />
            ) : previewError ? (
              <CenteredState icon="visibility_off" title={previewError} light />
            ) : previewHtml ? (
              <iframe
                key={selected?.entry_path}
                title="Design mockup preview"
                sandbox="allow-scripts allow-forms allow-pointer-lock allow-same-origin"
                srcDoc={previewHtml}
                onError={() => setPreviewError("Preview unavailable for this design mockup.")}
                style={{ width: "100%", height: "100%", border: 0, background: "#fff" }}
              />
            ) : (
              <CenteredState icon="visibility_off" title="No preview selected" light />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function CenteredState({
  icon,
  title,
  description,
  light,
}: {
  icon: string;
  title: string;
  description?: string;
  light?: boolean;
}) {
  return (
    <div
      style={{
        height: "100%",
        minHeight: 260,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        color: light ? "#36323c" : "var(--text-tertiary)",
      }}
    >
      <div style={{ textAlign: "center", maxWidth: 360, padding: 24 }}>
        <i className="material-symbols-outlined" style={{ fontSize: 30, marginBottom: 10 }}>
          {icon}
        </i>
        <div style={{ fontSize: 15, fontWeight: 700, marginBottom: description ? 6 : 0 }}>
          {title}
        </div>
        {description && <div style={{ fontSize: 12.5, lineHeight: 1.5 }}>{description}</div>}
      </div>
    </div>
  );
}

function formatBytes(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${Math.round(size / 1024)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}
