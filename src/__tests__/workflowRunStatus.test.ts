import { describe, expect, it } from "vitest";
import { describeWorkflowRun, getWorkflowRunStatusStyle } from "../lib/workflowRunStatus";

describe("getWorkflowRunStatusStyle", () => {
  it.each([
    ["success", "Success", "check_circle"],
    ["failure", "Failure", "error"],
    ["timed_out", "Timed out", "hourglass_bottom"],
    ["startup_failure", "Startup failure", "error"],
    ["cancelled", "Cancelled", "cancel"],
    ["skipped", "Skipped", "skip_next"],
    ["neutral", "Neutral", "remove"],
    ["stale", "Stale", "history"],
    ["action_required", "Action required", "warning"],
  ])("maps completed/%s to a distinct labeled style", (conclusion, label, icon) => {
    const style = getWorkflowRunStatusStyle("completed", conclusion);
    expect(style.label).toBe(label);
    expect(style.icon).toBe(icon);
  });

  it.each([
    ["in_progress", "In progress"],
    ["queued", "Queued"],
    ["waiting", "Waiting"],
    ["pending", "Pending"],
    ["requested", "Requested"],
  ])("maps non-completed status %s to a labeled style", (status, label) => {
    expect(getWorkflowRunStatusStyle(status, null).label).toBe(label);
  });

  it("reserves red exclusively for failing conclusions", () => {
    const failing = ["failure", "timed_out", "startup_failure"];
    const others = ["success", "cancelled", "skipped", "neutral", "stale", "action_required"];
    for (const conclusion of failing) {
      expect(getWorkflowRunStatusStyle("completed", conclusion).color).toBe("#ef4444");
    }
    for (const conclusion of others) {
      expect(getWorkflowRunStatusStyle("completed", conclusion).color).not.toBe("#ef4444");
    }
  });

  it("marks only in_progress as spinning", () => {
    expect(getWorkflowRunStatusStyle("in_progress", null).spin).toBe(true);
    expect(getWorkflowRunStatusStyle("queued", null).spin).toBe(false);
    expect(getWorkflowRunStatusStyle("completed", "success").spin).toBe(false);
  });

  it("falls back to a readable label for unknown values instead of hiding them", () => {
    expect(getWorkflowRunStatusStyle("completed", "some_new_state").label).toBe("some new state");
    expect(getWorkflowRunStatusStyle("brand_new_status", null).label).toBe("brand new status");
    expect(getWorkflowRunStatusStyle("completed", null).label).toBe("completed");
    expect(getWorkflowRunStatusStyle("", null).label).toBe("Unknown");
  });
});

describe("describeWorkflowRun", () => {
  it("produces an accessible text description", () => {
    expect(
      describeWorkflowRun({
        name: "CI Build",
        status: "completed",
        conclusion: "failure",
        head_branch: "main",
      }),
    ).toBe("CI Build, Failure, branch main");
  });
});
