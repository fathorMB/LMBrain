import { type CSSProperties, type ReactNode } from "react";

export interface FilterBarProps {
  children: ReactNode;
  ariaLabel?: string;
  style?: CSSProperties;
}

export function FilterBar({ children, ariaLabel = "Filters", style }: FilterBarProps) {
  return (
    <section
      aria-label={ariaLabel}
      style={{
        display: "flex",
        alignItems: "flex-end",
        flexWrap: "wrap",
        gap: 12,
        padding: 14,
        marginBottom: 14,
        border: "1px solid var(--border-secondary, #25202e)",
        borderRadius: 9,
        background: "var(--bg-secondary, #15111b)",
        ...style,
      }}
    >
      {children}
    </section>
  );
}

export interface FilterSelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: Array<{ value: string; label: string } | string>;
  ariaLabel?: string;
  allLabel?: string;
  style?: CSSProperties;
  className?: string;
}

export function FilterSelect({
  label,
  value,
  onChange,
  options,
  ariaLabel,
  allLabel,
  style,
  className = "app-select",
}: FilterSelectProps) {
  return (
    <label
      style={{
        display: "grid",
        gap: 6,
        flex: "1 1 120px",
        minWidth: 0,
        color: "var(--text-tertiary)",
        fontSize: "var(--text-xs)",
        fontWeight: 650,
        ...style,
      }}
    >
      {label}
      <select
        className={className}
        aria-label={ariaLabel || label}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        style={{
          minWidth: 0,
          height: 34,
          boxSizing: "border-box",
          border: "1px solid var(--border-primary, #332d3e)",
          borderRadius: 7,
          outline: "none",
          background: "var(--bg-tertiary, #1c1824)",
          color: "var(--text-primary)",
          colorScheme: "dark",
          padding: "0 9px",
          fontFamily: "inherit",
          fontSize: "var(--text-sm)",
        }}
      >
        {allLabel && <option value="all">{allLabel}</option>}
        {options.map((opt) => {
          const val = typeof opt === "string" ? opt : opt.value;
          const lbl = typeof opt === "string" ? opt : opt.label;
          return (
            <option key={val} value={val}>
              {lbl}
            </option>
          );
        })}
      </select>
    </label>
  );
}

export interface FilterSearchInputProps {
  label?: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  ariaLabel?: string;
  style?: CSSProperties;
}

export function FilterSearchInput({
  label = "Search",
  value,
  onChange,
  placeholder = "Filter items…",
  ariaLabel,
  style,
}: FilterSearchInputProps) {
  return (
    <label
      style={{
        display: "grid",
        gap: 6,
        flex: "1 1 240px",
        minWidth: 0,
        color: "var(--text-tertiary)",
        fontSize: "var(--text-xs)",
        fontWeight: 650,
        ...style,
      }}
    >
      {label}
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        aria-label={ariaLabel || label}
        style={{
          width: "100%",
          height: 34,
          boxSizing: "border-box",
          border: "1px solid var(--border-primary, #332d3e)",
          borderRadius: 7,
          outline: "none",
          background: "var(--bg-tertiary, #1c1824)",
          color: "var(--text-primary)",
          colorScheme: "dark",
          padding: "0 10px",
          fontFamily: "inherit",
          fontSize: "var(--text-sm)",
        }}
      />
    </label>
  );
}
