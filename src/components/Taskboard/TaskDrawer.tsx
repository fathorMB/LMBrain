import { useWorkspace } from "../../hooks/useWorkspace";

export function TaskDrawer() {
  const { state, closeTaskDrawer } = useWorkspace();
  const task = state.drawerTask;
  if (!task) return null;

  const statusColors: Record<string, { color: string; bg: string }> = {
    backlog: { color: "#9a949f", bg: "rgba(154,148,159,0.13)" },
    planned: { color: "#9a949f", bg: "rgba(154,148,159,0.13)" },
    "in-progress": { color: "#7fa8f5", bg: "rgba(91,141,239,0.14)" },
    review: { color: "#e0a23a", bg: "rgba(224,162,58,0.14)" },
    done: { color: "#5fc596", bg: "rgba(70,176,125,0.14)" },
    blocked: { color: "#f0a89f", bg: "rgba(224,88,74,0.14)" },
    cancelled: { color: "#6c6671", bg: "rgba(108,102,113,0.14)" },
  };
  const sc = statusColors[task.status] || statusColors.backlog;

  return (
    <>
      {/* Scrim */}
      <div
        onClick={closeTaskDrawer}
        style={{
          position: "fixed",
          inset: 0,
          background: "rgba(6,5,8,.55)",
          zIndex: 40,
        }}
      />
      {/* Drawer */}
      <div
        style={{
          position: "fixed",
          top: 0,
          right: 0,
          height: "100vh",
          width: 400,
          background: "#100e14",
          borderLeft: "1px solid #272330",
          zIndex: 41,
          display: "flex",
          flexDirection: "column",
          boxShadow: "-30px 0 70px -30px rgba(0,0,0,.8)",
        }}
      >
        {/* Header */}
        <div
          style={{
            flex: "none",
            padding: "16px 18px",
            borderBottom: "1px solid #201d26",
            display: "flex",
            alignItems: "center",
            gap: 11,
          }}
        >
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 12,
              color: "#bcaef6",
            }}
          >
            {task.id}
          </span>
          <span
            style={{
              display: "inline-flex",
              alignItems: "center",
              fontSize: 10.5,
              fontWeight: 700,
              color: sc.color,
              background: sc.bg,
              borderRadius: 5,
              padding: "2px 8px",
            }}
          >
            {task.status}
          </span>
          <div style={{ flex: 1 }} />
          <button
            onClick={closeTaskDrawer}
            style={{
              width: 30,
              height: 30,
              borderRadius: 8,
              background: "transparent",
              border: "1px solid #262330",
              color: "#9a949f",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "#1a1722";
              e.currentTarget.style.color = "var(--text-primary)";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.color = "#9a949f";
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 18 }}>
              close
            </i>
          </button>
        </div>

        {/* Content */}
        <div style={{ flex: 1, overflowY: "auto", padding: 18 }}>
          <h2
            style={{
              fontSize: 18,
              fontWeight: 700,
              letterSpacing: "-.01em",
              margin: "0 0 16px",
              lineHeight: 1.3,
            }}
          >
            {task.title}
          </h2>

          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 11,
              marginBottom: 20,
            }}
          >
            <DrawerRow label="Spec" value={task.spec || "—"} mono />
            <DrawerRow
              label="Priority"
              value={task.priority || "—"}
              bold
            />
            {task.area && <DrawerRow label="Area" value={task.area} mono />}
            <DrawerRow label="Updated" value={task.updated} />
          </div>

          {/* Criteria */}
          {task.criteria.length > 0 && (
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
                Acceptance criteria
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 8,
                  marginBottom: 20,
                }}
              >
                {task.criteria.map((c, i) => (
                  <div
                    key={i}
                    style={{
                      display: "flex",
                      gap: 9,
                      alignItems: "flex-start",
                    }}
                  >
                    <i
                      className="material-symbols-outlined"
                      style={{
                        fontSize: 17,
                        color: c.completed ? "var(--green)" : "#56525b",
                        flex: "none",
                      }}
                    >
                      {c.completed
                        ? "check_box"
                        : "check_box_outline_blank"}
                    </i>
                    <span
                      style={{
                        fontSize: 13,
                        color: "#c2bdc8",
                        lineHeight: 1.45,
                      }}
                    >
                      {c.text}
                    </span>
                  </div>
                ))}
              </div>
            </>
          )}

          {/* Dependencies */}
          {task.dependencies.length > 0 && (
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
                Dependencies
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: 6,
                  marginBottom: 20,
                }}
              >
                {task.dependencies.map((d, i) => (
                  <div
                    key={i}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 8,
                      fontSize: 12.5,
                      color: "#c2bdc8",
                      background: "var(--bg-tertiary)",
                      border: "1px solid var(--border-secondary)",
                      borderRadius: 8,
                      padding: "8px 11px",
                    }}
                  >
                    <i
                      className="material-symbols-outlined"
                      style={{ fontSize: 15, color: "#e0a23a" }}
                    >
                      link
                    </i>
                    {d}
                  </div>
                ))}
              </div>
            </>
          )}

          {/* Source file */}
          <div
            style={{
              fontSize: 10.5,
              letterSpacing: ".09em",
              textTransform: "uppercase",
              color: "#6c6671",
              fontWeight: 600,
              marginBottom: 9,
            }}
          >
            Source file
          </div>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 9,
              fontFamily: "var(--font-mono)",
              fontSize: 11.5,
              color: "#9a949f",
              background: "#0e0c12",
              border: "1px solid #221f29",
              borderRadius: 8,
              padding: "9px 11px",
              marginBottom: 20,
            }}
          >
            <i
              className="material-symbols-outlined"
              style={{ fontSize: 15, color: "#6c6671" }}
            >
              draft
            </i>
            <span
              style={{
                flex: 1,
                overflow: "hidden",
                textOverflow: "ellipsis",
              }}
            >
              {task.path}
            </span>
          </div>
        </div>
      </div>
    </>
  );
}

function DrawerRow({
  label,
  value,
  mono,
  bold,
}: {
  label: string;
  value: string;
  mono?: boolean;
  bold?: boolean;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
      <span
        style={{
          fontSize: 11.5,
          color: "#6c6671",
          width: 78,
          flex: "none",
        }}
      >
        {label}
      </span>
      <span
        style={{
          fontSize: mono ? 12 : 12.5,
          fontFamily: mono ? "var(--font-mono)" : "inherit",
          color: "#cfc9d6",
          fontWeight: bold ? 600 : 400,
        }}
      >
        {value}
      </span>
    </div>
  );
}
