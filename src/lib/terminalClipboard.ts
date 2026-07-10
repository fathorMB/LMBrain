export type TerminalClipboardAction = "copy" | "paste" | null;

export function terminalClipboardAction(
  event: Pick<KeyboardEvent, "type" | "key" | "ctrlKey" | "metaKey" | "shiftKey">,
  hasSelection: boolean
): TerminalClipboardAction {
  if (event.type !== "keydown") return null;
  const key = event.key.toLowerCase();
  const modifier = event.ctrlKey || event.metaKey;
  if (modifier && key === "c" && (event.metaKey || event.shiftKey || hasSelection)) {
    return "copy";
  }
  if (modifier && key === "v" && (event.metaKey || event.shiftKey)) {
    return "paste";
  }
  return null;
}
