export function getMigrationStatusLabelAndColor(status: string | undefined): { label: string; color: string } {
  switch (status) {
    case "up-to-date":
      return { label: "Up to date", color: "var(--green)" };
    case "migration-available":
      return { label: "Migration available", color: "var(--yellow)" };
    case "project-newer-than-app":
      return { label: "Project newer than app", color: "var(--red)" };
    case "unknown-project-version":
      return { label: "Unknown project version", color: "var(--text-muted)" };
    case "unknown-bundled-version":
      return { label: "Unknown bundled version", color: "var(--text-muted)" };
    case "migration-guidance-missing":
      return { label: "Guidance missing", color: "var(--yellow)" };
    default:
      return { label: "—", color: "var(--text-muted)" };
  }
}
