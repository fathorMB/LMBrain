import { useState } from "react";

const CODEX_BIN_SETTING = "lmbrain.codexBin";

export function SettingsView() {
  const [codexBin, setCodexBin] = useState(() => localStorage.getItem(CODEX_BIN_SETTING) ?? "");

  const updateCodexBin = (value: string) => {
    setCodexBin(value);
    const trimmed = value.trim();
    if (trimmed) {
      localStorage.setItem(CODEX_BIN_SETTING, trimmed);
    } else {
      localStorage.removeItem(CODEX_BIN_SETTING);
    }
  };

  return (
    <div style={{ overflowY: "auto", height: "100%" }}>
      <div style={{ maxWidth: 680, margin: "0 auto", padding: "24px 36px 70px" }}>
        <h1
          style={{
            fontSize: 24,
            fontWeight: 800,
            letterSpacing: "-.025em",
            margin: "0 0 22px",
          }}
        >
          Settings
        </h1>

        {/* Appearance */}
        <div
          style={{
            fontSize: 11,
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "#6c6671",
            fontWeight: 600,
            marginBottom: 11,
          }}
        >
          Appearance
        </div>
        <div
          style={{
            background: "var(--bg-tertiary)",
            border: "1px solid var(--border-secondary)",
            borderRadius: 12,
            padding: "6px 0",
            marginBottom: 22,
          }}
        >
          <SettingRow
            label="Theme"
            description="Dark is tuned for long sessions"
            options={["Dark", "Light"]}
            active="Dark"
          />
          <SettingRow
            label="Density"
            description="Compact rows for more on screen"
            options={["Cozy", "Dense"]}
            active="Dense"
          />
        </div>

        <div
          style={{
            background: "var(--bg-tertiary)",
            border: "1px solid var(--border-secondary)",
            borderRadius: 12,
            padding: "13px 16px",
            marginBottom: 22,
          }}
        >
          <div
            style={{
              fontSize: 13.5,
              fontWeight: 600,
              color: "var(--text-primary)",
              marginBottom: 5,
            }}
          >
            Codex executable
          </div>
          <div
            style={{
              fontSize: 11.5,
              color: "var(--text-tertiary)",
              marginBottom: 10,
            }}
          >
            Optional native Codex CLI path used before desktop auto-detection
          </div>
          <input
            value={codexBin}
            onChange={(event) => updateCodexBin(event.target.value)}
            placeholder="C:\\Users\\you\\AppData\\Local\\OpenAI\\Codex\\bin\\...\\codex.exe"
            style={{
              width: "100%",
              boxSizing: "border-box",
              borderRadius: 10,
              border: "1px solid #2c2538",
              background: "#0f0d14",
              color: "var(--text-primary)",
              padding: "10px 11px",
              fontSize: 12.5,
              outline: "none",
            }}
          />
        </div>

        {/* Agents */}
        <div
          style={{
            fontSize: 11,
            letterSpacing: ".09em",
            textTransform: "uppercase",
            color: "#6c6671",
            fontWeight: 600,
            marginBottom: 11,
          }}
        >
          Agents
        </div>
        <div
          style={{
            background: "var(--bg-tertiary)",
            border: "1px solid var(--border-secondary)",
            borderRadius: 12,
            padding: "13px 16px",
            display: "flex",
            alignItems: "center",
            gap: 14,
          }}
        >
          <div style={{ flex: 1 }}>
            <div
              style={{
                fontSize: 13.5,
                fontWeight: 600,
                color: "var(--text-primary)",
              }}
            >
              Auto-start agents
            </div>
            <div
              style={{
                fontSize: 11.5,
                color: "var(--text-tertiary)",
              }}
            >
              LMBrain never launches agents automatically — this stays off by
              design.
            </div>
          </div>
          <div
            style={{
              width: 38,
              height: 22,
              borderRadius: 12,
              background: "#26222d",
              border: "1px solid #322d3a",
              position: "relative",
              flex: "none",
            }}
          >
            <div
              style={{
                position: "absolute",
                top: 2,
                left: 2,
                width: 16,
                height: 16,
                borderRadius: "50%",
                background: "#6c6671",
              }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}

function SettingRow({
  label,
  description,
  options,
  active,
}: {
  label: string;
  description: string;
  options: string[];
  active: string;
}) {
  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        gap: 14,
        padding: "13px 16px",
        borderBottom: "1px solid #201d26",
      }}
    >
      <div style={{ flex: 1 }}>
        <div
          style={{
            fontSize: 13.5,
            fontWeight: 600,
            color: "var(--text-primary)",
          }}
        >
          {label}
        </div>
        <div
          style={{
            fontSize: 11.5,
            color: "var(--text-tertiary)",
          }}
        >
          {description}
        </div>
      </div>
      <div
        style={{
          display: "flex",
          background: "#100e14",
          border: "1px solid #262330",
          borderRadius: 8,
          padding: 2,
        }}
      >
        {options.map((opt) => (
          <span
            key={opt}
            style={{
              fontSize: 12,
              fontWeight: opt === active ? 600 : 500,
              color:
                opt === active ? "var(--text-primary)" : "var(--text-tertiary)",
              background:
                opt === active ? "var(--bg-active)" : "transparent",
              borderRadius: 6,
              padding: "4px 12px",
              cursor: "pointer",
            }}
          >
            {opt}
          </span>
        ))}
      </div>
    </div>
  );
}
