/**
 * Shared status/conclusion presentation for GitHub Actions workflow runs.
 *
 * Every status and conclusion GitHub can return maps to a distinct,
 * text-labeled style; unknown values fall back to a neutral style with the
 * raw label so no run is ever hidden or mislabeled. Red is reserved for
 * failing conclusions so failures stay instantly identifiable.
 */

export interface WorkflowRunStatusStyle {
  label: string;
  color: string;
  bg: string;
  icon: string;
  spin: boolean;
}

const style = (
  label: string,
  color: string,
  icon: string,
  spin = false,
): WorkflowRunStatusStyle => ({
  label,
  color,
  bg: `${color}1f`,
  icon,
  spin,
});

const CONCLUSION_STYLES: Record<string, WorkflowRunStatusStyle> = {
  success: style("Success", "#10b981", "check_circle"),
  failure: style("Failure", "#ef4444", "error"),
  timed_out: style("Timed out", "#ef4444", "hourglass_bottom"),
  startup_failure: style("Startup failure", "#ef4444", "error"),
  cancelled: style("Cancelled", "#9ca3af", "cancel"),
  skipped: style("Skipped", "#9ca3af", "skip_next"),
  neutral: style("Neutral", "#9ca3af", "remove"),
  stale: style("Stale", "#9ca3af", "history"),
  action_required: style("Action required", "#f59e0b", "warning"),
};

const STATUS_STYLES: Record<string, WorkflowRunStatusStyle> = {
  in_progress: style("In progress", "#6366f1", "progress_activity", true),
  queued: style("Queued", "#6366f1", "pending"),
  waiting: style("Waiting", "#6366f1", "pending"),
  pending: style("Pending", "#6366f1", "pending"),
  requested: style("Requested", "#6366f1", "pending"),
};

const fallback = (raw: string): WorkflowRunStatusStyle =>
  style(raw.replace(/_/g, " ") || "Unknown", "#9ca3af", "help");

export function getWorkflowRunStatusStyle(
  status: string,
  conclusion: string | null,
): WorkflowRunStatusStyle {
  if (status === "completed") {
    if (!conclusion) return fallback("completed");
    return CONCLUSION_STYLES[conclusion] ?? fallback(conclusion);
  }
  return STATUS_STYLES[status] ?? fallback(status);
}

export function describeWorkflowRun(run: {
  name: string;
  status: string;
  conclusion: string | null;
  head_branch: string;
}): string {
  const state = getWorkflowRunStatusStyle(run.status, run.conclusion).label;
  return `${run.name}, ${state}, branch ${run.head_branch}`;
}
