import { describe, expect, it } from "vitest";
import { sessionReducer } from "../context/WorkspaceContext";
import type { SessionInfo } from "../types";

const sessionA: SessionInfo = {
  id: "a",
  label: "Session A",
  host: "claude",
  route: "native",
  model: null,
  status: "running",
  exit_code: null,
};

const sessionB: SessionInfo = {
  id: "b",
  label: "Session B",
  host: "codex",
  route: "native",
  model: null,
  status: "running",
  exit_code: null,
};

const sessionC: SessionInfo = {
  id: "c",
  label: "Session C",
  host: "claude",
  route: "ollama",
  model: "llama3",
  status: "exited",
  exit_code: 0,
};

describe("sessionReducer (production code)", () => {
  it("ADD_SESSION adds a tab and makes it active", () => {
    const next = sessionReducer(
      { sessions: [], activeSessionId: null },
      { type: "ADD_SESSION", session: sessionA }
    );
    expect(next.sessions).toHaveLength(1);
    expect(next.sessions[0].id).toBe("a");
    expect(next.activeSessionId).toBe("a");
  });

  it("ADD_SESSION appends and activates when sessions already exist", () => {
    const next = sessionReducer(
      { sessions: [sessionA], activeSessionId: "a" },
      { type: "ADD_SESSION", session: sessionB }
    );
    expect(next.sessions).toHaveLength(2);
    expect(next.sessions[1].id).toBe("b");
    expect(next.activeSessionId).toBe("b");
  });

  it("UPDATE_SESSION patches the matching session and preserves tab order and active tab", () => {
    const next = sessionReducer(
      { sessions: [sessionA, sessionB, sessionC], activeSessionId: "a" },
      {
        type: "UPDATE_SESSION",
        id: "b",
        patch: { status: "exited", exit_code: 42 },
      }
    );

    expect(next.sessions.map((session) => session.id)).toEqual(["a", "b", "c"]);
    expect(next.sessions[1]).toMatchObject({
      id: "b",
      status: "exited",
      exit_code: 42,
    });
    expect(next.activeSessionId).toBe("a");
  });

  it("REMOVE_SESSION on active tab selects the previous neighbor", () => {
    const next = sessionReducer(
      { sessions: [sessionA, sessionB, sessionC], activeSessionId: "b" },
      { type: "REMOVE_SESSION", id: "b" }
    );
    expect(next.sessions).toHaveLength(2);
    expect(next.sessions.find((s) => s.id === "b")).toBeUndefined();
    expect(next.activeSessionId).toBe("a");
  });

  it("REMOVE_SESSION on first active tab selects the next (now first)", () => {
    const next = sessionReducer(
      { sessions: [sessionA, sessionB], activeSessionId: "a" },
      { type: "REMOVE_SESSION", id: "a" }
    );
    expect(next.sessions).toHaveLength(1);
    expect(next.activeSessionId).toBe("b");
  });

  it("REMOVE_SESSION on last active tab selects the previous", () => {
    const next = sessionReducer(
      { sessions: [sessionA, sessionB], activeSessionId: "b" },
      { type: "REMOVE_SESSION", id: "b" }
    );
    expect(next.sessions).toHaveLength(1);
    expect(next.activeSessionId).toBe("a");
  });

  it("REMOVE_SESSION on last remaining tab sets activeSessionId to null", () => {
    const next = sessionReducer(
      { sessions: [sessionA], activeSessionId: "a" },
      { type: "REMOVE_SESSION", id: "a" }
    );
    expect(next.sessions).toHaveLength(0);
    expect(next.activeSessionId).toBeNull();
  });

  it("REMOVE_SESSION on non-active tab preserves activeSessionId", () => {
    const next = sessionReducer(
      { sessions: [sessionA, sessionB], activeSessionId: "a" },
      { type: "REMOVE_SESSION", id: "b" }
    );
    expect(next.sessions).toHaveLength(1);
    expect(next.activeSessionId).toBe("a");
  });

  it("SET_SESSIONS preserves activeSessionId when it exists in the new list", () => {
    const next = sessionReducer(
      { sessions: [sessionA, sessionB], activeSessionId: "a" },
      { type: "SET_SESSIONS", sessions: [sessionA, sessionB, sessionC] }
    );
    expect(next.activeSessionId).toBe("a");
  });

  it("SET_SESSIONS falls back to first session when activeSessionId is stale", () => {
    const next = sessionReducer(
      { sessions: [sessionA, sessionB], activeSessionId: "a" },
      { type: "SET_SESSIONS", sessions: [sessionB, sessionC] }
    );
    expect(next.activeSessionId).toBe("b");
  });

  it("SET_SESSIONS sets null when the new list is empty", () => {
    const next = sessionReducer(
      { sessions: [sessionA], activeSessionId: "a" },
      { type: "SET_SESSIONS", sessions: [] }
    );
    expect(next.sessions).toHaveLength(0);
    expect(next.activeSessionId).toBeNull();
  });

  it("SET_SESSIONS with null activeSessionId picks the first session", () => {
    const next = sessionReducer(
      { sessions: [], activeSessionId: null },
      { type: "SET_SESSIONS", sessions: [sessionA, sessionB] }
    );
    expect(next.activeSessionId).toBe("a");
  });
});
