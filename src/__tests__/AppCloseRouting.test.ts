import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it, vi } from "vitest";
import { resolveOpenSessions, routeWindowCloseRequest } from "../lib/windowClose";
import type { SessionInfo } from "../types";

const session = (id: string, status: SessionInfo["status"]): SessionInfo => ({
  id,
  label: id,
  host: "codex",
  route: "native",
  model: null,
  status,
  exit_code: null,
});

describe("routeWindowCloseRequest", () => {
  it("has the Tauri capability required by the explicit close path", () => {
    const capabilities = JSON.parse(
      readFileSync(
        resolve(process.cwd(), "src-tauri/capabilities/default.json"),
        "utf8",
      ),
    ) as { permissions: string[] };

    expect(capabilities.permissions).toContain("core:window:allow-destroy");
  });

  it("destroys the window immediately when no sessions are active", async () => {
    const destroy = vi.fn().mockResolvedValue(undefined);
    const showConfirmation = vi.fn();

    await routeWindowCloseRequest({
      openSessionCount: 0,
      destroy,
      showConfirmation,
    });

    expect(destroy).toHaveBeenCalledTimes(1);
    expect(showConfirmation).not.toHaveBeenCalled();
  });

  it("shows the in-app confirmation without destroying when session tabs are open", async () => {
    const destroy = vi.fn().mockResolvedValue(undefined);
    const showConfirmation = vi.fn();

    await routeWindowCloseRequest({
      openSessionCount: 2,
      destroy,
      showConfirmation,
    });

    expect(showConfirmation).toHaveBeenCalledTimes(1);
    expect(destroy).not.toHaveBeenCalled();
  });

  it("uses the authoritative backend session list instead of stale local state", async () => {
    const backendSessions = [session("running", "running")];

    await expect(
      resolveOpenSessions({
        listSessions: vi.fn().mockResolvedValue(backendSessions),
        fallbackSessions: [],
      }),
    ).resolves.toEqual(backendSessions);
  });

  it("fails safe to local open tabs when the backend list is unavailable", async () => {
    const localSessions = [session("exited-tab", "exited")];

    await expect(
      resolveOpenSessions({
        listSessions: vi.fn().mockRejectedValue(new Error("backend unavailable")),
        fallbackSessions: localSessions,
      }),
    ).resolves.toEqual(localSessions);
  });
});
