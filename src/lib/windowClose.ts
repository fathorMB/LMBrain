import type { SessionInfo } from "../types";

export async function resolveOpenSessions({
  listSessions,
  fallbackSessions,
}: {
  listSessions: () => Promise<SessionInfo[]>;
  fallbackSessions: SessionInfo[];
}): Promise<SessionInfo[]> {
  try {
    return await listSessions();
  } catch {
    return fallbackSessions;
  }
}

export async function routeWindowCloseRequest({
  openSessionCount,
  destroy,
  showConfirmation,
}: {
  openSessionCount: number;
  destroy: () => Promise<void>;
  showConfirmation: () => void;
}) {
  if (openSessionCount === 0) {
    await destroy();
    return;
  }
  showConfirmation();
}
