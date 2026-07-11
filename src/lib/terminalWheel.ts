export type TerminalBufferType = "normal" | "alternate";

export function shouldDelegateTerminalWheel(
  bufferType: TerminalBufferType,
  event: Pick<WheelEvent, "ctrlKey" | "metaKey">
) {
  return bufferType === "alternate" || event.ctrlKey || event.metaKey;
}

export function terminalWheelRows(deltaY: number) {
  return Math.max(1, Math.ceil(Math.abs(deltaY) / 36));
}
