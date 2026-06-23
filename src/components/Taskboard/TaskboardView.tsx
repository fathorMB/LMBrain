import { useEffect, useState } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { getTasks, getDiagnostics } from "../../lib/commands";
import type { Task, TaskStatus } from "../../types";

/** Folder/frontmatter status mismatch surfaced on a card, keyed by task id. */
interface StatusMismatch {
  /** The status declared in the file's frontmatter (differs from its folder). */
  declared: string;
  /** The full diagnostic message, used as the tooltip. */
  message: string;
}

const normalizePath = (p: string) => p.replace(/\\/g, "/").toLowerCase();

const COLUMNS: { status: TaskStatus; label: string; color: string }[] = [
  { status: "backlog", label: "Backlog", color: "#6c6671" },
  { status: "planned", label: "Planned", color: "#8a8d99" },
  { status: "in-progress", label: "In Progress", color: "#5b8def" },
  { status: "review", label: "Review", color: "#e0a23a" },
  { status: "done", label: "Done", color: "#46b07d" },
  { status: "blocked", label: "Blocked", color: "#e0584a" },
  { status: "cancelled", label: "Cancelled", color: "#6c6671" },
];

export function TaskboardView() {
  const { state, dispatch, openTaskDrawer } = useWorkspace();
  const [mismatches, setMismatches] = useState<Record<string, StatusMismatch>>({});

  useEffect(() => {
    Promise.all([getTasks(), getDiagnostics()])
      .then(([tasks, diagnostics]) => {
        dispatch({ type: "SET_TASKS", tasks });

        // A task's column is derived from its folder; the file's `status:`
        // frontmatter must agree. When they diverge the backend already emits a
        // "Status mismatch" warning — surface it on the card so a half-applied
        // transition (field updated but file not moved, or vice versa) is visible
        // instead of looking like the task silently never changed state.
        const map: Record<string, StatusMismatch> = {};
        for (const task of tasks) {
          const diag = diagnostics.find(
            (d) =>
              d.severity === "warning" &&
              d.message.includes("Status mismatch") &&
              d.path != null &&
              normalizePath(task.path).endsWith(normalizePath(d.path))
          );
          if (diag) {
            const declared =
              /frontmatter status is '([^']+)'/.exec(diag.message)?.[1] ?? "?";
            map[task.id] = { declared, message: diag.message };
          }
        }
        setMismatches(map);
      })
      .catch(console.error);
  }, [dispatch]);

  const tasksByStatus = (status: TaskStatus) =>
    state.tasks.filter((t) => t.status === status);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {/* Header */}
      <div
        style={{
          flex: "none",
          padding: "20px 24px 14px",
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            marginBottom: 14,
          }}
        >
          <h1
            style={{
              fontSize: 24,
              fontWeight: 800,
              letterSpacing: "-.025em",
              margin: 0,
            }}
          >
            Taskboard
          </h1>
          <div
            style={{
              fontSize: 12,
              color: "var(--text-tertiary)",
              display: "flex",
              alignItems: "center",
              gap: 7,
            }}
          >
            <i
              className="material-symbols-outlined"
              style={{ fontSize: 15, color: "var(--green)" }}
            >
              cloud_done
            </i>
            backed by{" "}
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: 11.5,
                color: "#9a949f",
              }}
            >
              .lmbrain/tasks/&lt;status&gt;/*.md
            </span>
          </div>
        </div>
        <div
          style={{
            fontSize: 11.5,
            color: "#56525b",
            display: "flex",
            alignItems: "center",
            gap: 6,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
            visibility
          </i>
          Read-only view · status changes happen in Markdown
        </div>
      </div>

      {/* Columns */}
      <div
        style={{
          flex: 1,
          overflowX: "auto",
          overflowY: "hidden",
          padding: "16px 24px",
        }}
      >
        <div
          style={{
            display: "flex",
            gap: 14,
            height: "100%",
            minWidth: "max-content",
          }}
        >
          {COLUMNS.map((col) => {
            const tasks = tasksByStatus(col.status);
            const isBlocked = col.status === "blocked";

            return (
              <div
                key={col.status}
                style={{
                  width: 262,
                  flex: "none",
                  display: "flex",
                  flexDirection: "column",
                  minHeight: 0,
                  ...(isBlocked
                    ? {
                        background: "rgba(224,88,74,.05)",
                        border: "1px solid rgba(224,88,74,.2)",
                        borderRadius: 13,
                        padding: "11px 9px",
                      }
                    : {}),
                }}
              >
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    padding: "0 4px 11px",
                  }}
                >
                  <span
                    style={{
                      width: 9,
                      height: 9,
                      borderRadius: "50%",
                      background: col.color,
                    }}
                  />
                  <span
                    style={{
                      fontSize: 12.5,
                      fontWeight: 700,
                      color: isBlocked ? "#f0a89f" : "var(--text-primary)",
                    }}
                  >
                    {col.label}
                  </span>
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: 11,
                      color: isBlocked ? "#9a6b66" : "#56525b",
                    }}
                  >
                    {tasks.length}
                  </span>
                </div>
                <div
                  style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 9,
                    overflowY: "auto",
                    paddingRight: 2,
                  }}
                >
                  {tasks.map((task) => (
                    <TaskCard
                      key={task.id}
                      task={task}
                      mismatch={mismatches[task.id]}
                      onClick={() => openTaskDrawer(task)}
                    />
                  ))}
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

function TaskCard({
  task,
  mismatch,
  onClick,
}: {
  task: Task;
  mismatch?: StatusMismatch;
  onClick: () => void;
}) {
  const priorityColors: Record<string, { color: string; bg: string }> = {
    High: { color: "#e0a23a", bg: "rgba(224,162,58,0.13)" },
    Med: { color: "#8a8d99", bg: "rgba(139,141,152,0.13)" },
    Low: { color: "#7d7886", bg: "rgba(125,120,134,0.12)" },
  };
  const pc = priorityColors[task.priority || ""] || priorityColors.Med;
  const isMalformed = !!task.malformed;

  return (
    <div
      onClick={onClick}
      style={{
        background: "var(--bg-tertiary)",
        border: isMalformed ? "1px solid #e0584a" : "1px solid #262330",
        borderRadius: 11,
        padding: "12px 13px",
        cursor: "pointer",
        display: "flex",
        flexDirection: "column",
        gap: 8,
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor = isMalformed ? "#f06f60" : "#3a3446";
        e.currentTarget.style.background = "#181520";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor = isMalformed ? "#e0584a" : "#262330";
        e.currentTarget.style.background = "var(--bg-tertiary)";
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span
          style={{
            fontFamily: "var(--font-mono)",
            fontSize: 11,
            color: "var(--text-tertiary)",
          }}
        >
          {task.id}
        </span>
        {isMalformed ? (
          <span
            style={{
              fontSize: 10,
              fontWeight: 700,
              color: "#e0584a",
              background: "rgba(224,88,74,0.13)",
              borderRadius: 5,
              padding: "2px 7px",
              letterSpacing: "0.03em",
              display: "inline-flex",
              alignItems: "center",
              gap: 3,
            }}
          >
            <i className="material-symbols-outlined" style={{ fontSize: 11 }}>warning</i>
            MALFORMED
          </span>
        ) : (
          task.priority && (
            <span
              style={{
                fontSize: 10,
                fontWeight: 700,
                color: pc.color,
                background: pc.bg,
                borderRadius: 5,
                padding: "2px 7px",
                letterSpacing: "0.03em",
              }}
            >
              {task.priority}
            </span>
          )
        )}
      </div>
      <div
        style={{
          fontSize: 13,
          fontWeight: 600,
          lineHeight: 1.35,
          color: "var(--text-primary)",
        }}
      >
        {task.title}
      </div>
      {task.block_reason && (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontSize: 11,
            color: "#f0a89f",
            background: "rgba(224,88,74,0.1)",
            border: "1px solid rgba(224,88,74,0.22)",
            borderRadius: 6,
            padding: "4px 8px",
            width: "max-content",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
            link_off
          </i>
          {task.block_reason}
        </div>
      )}
      {mismatch && (
        <div
          title={mismatch.message}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 6,
            fontSize: 11,
            color: "#e0a23a",
            background: "rgba(224,162,58,0.1)",
            border: "1px solid rgba(224,162,58,0.28)",
            borderRadius: 6,
            padding: "4px 8px",
            width: "max-content",
            maxWidth: "100%",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 13 }}>
            sync_problem
          </i>
          status: {mismatch.declared} ≠ folder: {task.status}
        </div>
      )}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          marginTop: 1,
        }}
      >
        {task.area && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 10.5,
              color: "#9a949f",
              background: "#1a1722",
              borderRadius: 5,
              padding: "2px 6px",
            }}
          >
            {task.area}
          </span>
        )}
        {task.spec && (
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 10.5,
              color: "#bcaef6",
            }}
          >
            {task.spec}
          </span>
        )}
        <span style={{ flex: 1 }} />
        <span
          style={{
            fontSize: 10.5,
            color: "#56525b",
            whiteSpace: "nowrap",
          }}
        >
          {task.updated}
        </span>
      </div>
    </div>
  );
}
