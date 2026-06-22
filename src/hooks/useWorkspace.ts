import { useContext } from "react";
import { WorkspaceContext } from "../context/WorkspaceContext";
import type { WorkspaceContextValue } from "../context/WorkspaceContext";

export function useWorkspace(): WorkspaceContextValue {
  const ctx = useContext(WorkspaceContext);
  if (!ctx) throw new Error("useWorkspace must be used within WorkspaceProvider");
  return ctx;
}
