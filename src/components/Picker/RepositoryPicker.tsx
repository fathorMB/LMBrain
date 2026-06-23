import { useState, useEffect } from "react";
import { useWorkspace } from "../../hooks/useWorkspace";
import { listRecentWorkspaces } from "../../lib/commands";
import type { WorkspaceSummary } from "../../types";

export function RepositoryPicker() {
  const { state, openWorkspace } = useWorkspace();
  const [recentItems, setRecentItems] = useState<WorkspaceSummary[]>([]);

  useEffect(() => {
    listRecentWorkspaces().then(setRecentItems).catch(() => {});
  }, []);

  const handleOpenWorkspace = async () => {
    // Use the Tauri dialog to pick a folder
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Choose a repository folder",
      });
      if (selected) {
        await openWorkspace(selected as string);
      }
    } catch (err) {
      console.error("Failed to open folder picker:", err);
    }
  };

  const handleOpenRecent = async (path: string) => {
    await openWorkspace(path);
  };

  const healthDot = (health: string) => {
    switch (health) {
      case "ok":
        return "#46b07d";
      case "warn":
        return "#e0a23a";
      default:
        return "#6c6671";
    }
  };

  return (
    <div
      style={{
        height: "100vh",
        width: "100vw",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: 36,
        background:
          "radial-gradient(1100px 620px at 26% -12%, #1b1624 0%, #0b0a0d 56%)",
      }}
    >
      <div style={{ width: 1010, maxWidth: "100%" }}>
        {/* Header */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 12,
            marginBottom: 22,
          }}
        >
          <div
            style={{
              width: 36,
              height: 36,
              borderRadius: 10,
              background: "linear-gradient(152deg,#9384f8,#6a4ff0)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              boxShadow: "0 6px 20px -4px rgba(106,79,240,.55)",
            }}
          >
            <i
              className="material-symbols-outlined"
              style={{ fontSize: 21, color: "#fff" }}
            >
              neurology
            </i>
          </div>
          <div>
            <div
              style={{
                fontSize: 16,
                fontWeight: 700,
                letterSpacing: "-.01em",
              }}
            >
              LMBrain
            </div>
            <div
              style={{
                fontSize: 11.5,
                color: "#6c6671",
                fontFamily: "var(--font-mono)",
              }}
            >
              local-first project brain
            </div>
          </div>
        </div>

        {/* Main card */}
        <div
          style={{
            display: "grid",
            gridTemplateColumns: "1.04fr .96fr",
            background: "#201d27",
            border: "1px solid #2a2731",
            borderRadius: 16,
            overflow: "hidden",
            boxShadow: "0 36px 90px -34px rgba(0,0,0,.85)",
          }}
        >
          {/* Left: Recent workspaces */}
          <div
            style={{
              background: "#131117",
              padding: "30px 30px 26px",
              borderRight: "1px solid #232029",
            }}
          >
            <h1
              style={{
                fontSize: 26,
                fontWeight: 700,
                letterSpacing: "-.022em",
                margin: "0 0 9px",
              }}
            >
              Open a project brain
            </h1>
            <p
              style={{
                fontSize: 13.5,
                lineHeight: 1.55,
                color: "#9a949f",
                margin: "0 0 21px",
              }}
            >
              Point LMBrain at a local repository containing a{" "}
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  color: "#bcaef6",
                  fontSize: 12.5,
                }}
              >
                .lmbrain
              </span>{" "}
              directory. Files are read in place — your repo is never copied or
              uploaded.
            </p>

            <button
              onClick={handleOpenWorkspace}
              style={{
                width: "100%",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                gap: 9,
                background: "linear-gradient(180deg,#8676f7,#6e5bf2)",
                color: "#fff",
                border: "none",
                borderRadius: 10,
                padding: 12,
                fontSize: 13.5,
                fontWeight: 600,
                cursor: "pointer",
                boxShadow: "0 8px 20px -7px rgba(110,91,242,.75)",
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.filter = "brightness(1.08)";
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.filter = "none";
              }}
            >
              <i className="material-symbols-outlined" style={{ fontSize: 19 }}>
                folder_open
              </i>
              Choose repository folder…
            </button>

            <div
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "space-between",
                margin: "26px 0 11px",
              }}
            >
              <span
                style={{
                  fontSize: 10.5,
                  letterSpacing: ".09em",
                  textTransform: "uppercase",
                  color: "#6c6671",
                  fontWeight: 600,
                }}
              >
                Recent
              </span>
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: 11,
                  color: "#56525b",
                }}
              >
                {recentItems.length} workspaces
              </span>
            </div>

            <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
              {recentItems.map((item) => (
                <div
                  key={item.path}
                  onClick={() => handleOpenRecent(item.path)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 11,
                    padding: "9px 10px",
                    borderRadius: 9,
                    cursor: "pointer",
                    border: "1px solid transparent",
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = "#1b1822";
                    e.currentTarget.style.borderColor = "#2b2833";
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = "transparent";
                    e.currentTarget.style.borderColor = "transparent";
                  }}
                >
                  <div
                    style={{
                      width: 8,
                      height: 8,
                      borderRadius: "50%",
                      flex: "none",
                      background: healthDot(item.health),
                      boxShadow: `0 0 9px ${healthDot(item.health)}`,
                    }}
                  />
                  <div style={{ minWidth: 0, flex: 1 }}>
                    <div
                      style={{
                        fontSize: 13,
                        fontWeight: 600,
                        color: "var(--text-primary)",
                      }}
                    >
                      {item.name}
                    </div>
                    <div
                      style={{
                        fontFamily: "var(--font-mono)",
                        fontSize: 11,
                        color: "#6c6671",
                        whiteSpace: "nowrap",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                      }}
                    >
                      {item.path}
                    </div>
                  </div>
                  <div
                    style={{
                      fontSize: 11,
                      color: "#6c6671",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {item.last_opened}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Right: Workspace preview */}
          <div
            style={{
              padding: "26px 28px",
              display: "flex",
              flexDirection: "column",
              minHeight: 392,
            }}
          >
            {state.currentWorkspace ? (
              <WorkspacePreview />
            ) : (
              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  height: "100%",
                  textAlign: "center",
                  alignItems: "center",
                  justifyContent: "center",
                }}
              >
                <div
                  style={{
                    width: 48,
                    height: 48,
                    borderRadius: 13,
                    background: "rgba(224,162,58,.12)",
                    border: "1px solid rgba(224,162,58,.28)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    marginBottom: 15,
                  }}
                >
                  <i
                    className="material-symbols-outlined"
                    style={{ fontSize: 24, color: "#e0a23a" }}
                  >
                    folder_off
                  </i>
                </div>
                <h2
                  style={{
                    fontSize: 17,
                    fontWeight: 700,
                    margin: "0 0 7px",
                    color: "var(--text-primary)",
                  }}
                >
                  Select a workspace
                </h2>
                <p
                  style={{
                    fontSize: 13,
                    lineHeight: 1.55,
                    color: "#9a949f",
                    margin: 0,
                    maxWidth: 300,
                  }}
                >
                  Choose a repository folder or select one from your recent
                  workspaces to get started.
                </p>
              </div>
            )}
          </div>
        </div>

        <div
          style={{
            textAlign: "center",
            marginTop: 16,
            fontSize: 11.5,
            color: "#56525b",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 7,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 15 }}>
            lock
          </i>
          Everything stays on this machine · No account required · Markdown is
          the source of truth
        </div>
      </div>
    </div>
  );
}

function WorkspacePreview() {
  const { state, initializeWorkspaceKit } = useWorkspace();
  const ws = state.currentWorkspace;
  if (!ws) return null;

  if (ws.health === "none") {
    return (
      <div style={{ display: "flex", flexDirection: "column", height: "100%", justifyContent: "center" }}>
        <div
          style={{
            width: 48,
            height: 48,
            borderRadius: 13,
            background: "rgba(224,162,58,.12)",
            border: "1px solid rgba(224,162,58,.28)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            marginBottom: 15,
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 24, color: "#e0a23a" }}>neurology</i>
        </div>
        <h2 style={{ fontSize: 17, fontWeight: 700, margin: "0 0 7px", color: "var(--text-primary)" }}>
          Initialize this project brain?
        </h2>
        <p style={{ fontSize: 13, lineHeight: 1.55, color: "#9a949f", margin: "0 0 18px", maxWidth: 350 }}>
          <span style={{ fontFamily: "var(--font-mono)", color: "#bcaef6" }}>{ws.path}</span> does not contain an LMBrain kit. Initializing creates a new <span style={{ fontFamily: "var(--font-mono)", color: "#bcaef6" }}>.lmbrain/</span> directory in this repository; existing files are not changed.
        </p>
        <button
          type="button"
          onClick={() => initializeWorkspaceKit(ws.path)}
          style={{
            alignSelf: "flex-start",
            display: "flex",
            alignItems: "center",
            gap: 8,
            background: "linear-gradient(180deg,#8676f7,#6e5bf2)",
            color: "#fff",
            border: "none",
            borderRadius: 10,
            padding: "11px 14px",
            fontSize: 13,
            fontWeight: 600,
            cursor: "pointer",
          }}
        >
          <i className="material-symbols-outlined" style={{ fontSize: 18 }}>add_circle</i>
          Initialize LMBrain kit
        </button>
      </div>
    );
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <span
        style={{
          fontSize: 10.5,
          letterSpacing: ".09em",
          textTransform: "uppercase",
          color: "#6c6671",
          fontWeight: 600,
        }}
      >
        Selected workspace
      </span>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 12,
          margin: "13px 0 20px",
        }}
      >
        <div
          style={{
            width: 40,
            height: 40,
            borderRadius: 11,
            background: "linear-gradient(150deg,#2c2738,#211d2b)",
            border: "1px solid #36303f",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontFamily: "var(--font-mono)",
            fontWeight: 600,
            color: "#a596f5",
            fontSize: 17,
          }}
        >
          {ws.name.charAt(0).toUpperCase()}
        </div>
        <div style={{ minWidth: 0 }}>
          <div
            style={{
              fontSize: 16,
              fontWeight: 700,
              letterSpacing: "-.01em",
              color: "var(--text-primary)",
            }}
          >
            {ws.name}
          </div>
          <div
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: 11.5,
              color: "#6c6671",
              whiteSpace: "nowrap",
              overflow: "hidden",
              textOverflow: "ellipsis",
            }}
          >
            {ws.path}
          </div>
        </div>
      </div>

      <div
        style={{
          display: "grid",
          gridTemplateColumns: "1fr 1fr",
          gap: 8,
          marginBottom: 16,
        }}
      >
        <StatBox value={ws.spec_count} label="specifications" />
        <StatBox value={ws.decision_count} label="decisions" />
        <StatBox value={ws.agent_count} label="agent profiles" />
      </div>

      <div
        style={{
          display: "flex",
          flexDirection: "column",
          gap: 8,
          fontSize: 12.5,
          color: "#b6b1bb",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <i
            className="material-symbols-outlined"
            style={{ fontSize: 16, color: "var(--green)" }}
          >
            check_circle
          </i>
          <span
            style={{
              fontFamily: "var(--font-mono)",
              color: "#bcaef6",
              fontSize: 12,
            }}
          >
            .lmbrain/
          </span>{" "}
          detected · {ws.health === "ok" ? "readable" : ws.health}
        </div>
        {ws.kit_version && (
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <i
              className="material-symbols-outlined"
              style={{ fontSize: 16, color: "var(--green)" }}
            >
              check_circle
            </i>
            Kit version {ws.kit_version}
          </div>
        )}
        {ws.diagnostics.map((d, i) => (
          <div
            key={i}
            style={{ display: "flex", alignItems: "center", gap: 8 }}
          >
            <i
              className="material-symbols-outlined"
              style={{
                fontSize: 16,
                color:
                  d.severity === "error"
                    ? "var(--red)"
                    : "var(--yellow)",
              }}
            >
              {d.severity === "error" ? "error" : "warning"}
            </i>
            <span style={{ fontSize: 12 }}>{d.message}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

function StatBox({ value, label }: { value: number; label: string }) {
  return (
    <div
      style={{
        background: "#141217",
        border: "1px solid #232029",
        borderRadius: 9,
        padding: "10px 12px",
      }}
    >
      <div
        style={{
          fontSize: 19,
          fontWeight: 700,
          fontFamily: "var(--font-mono)",
        }}
      >
        {value}
      </div>
      <div style={{ fontSize: 11, color: "#6c6671" }}>{label}</div>
    </div>
  );
}
