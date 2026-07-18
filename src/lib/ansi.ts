// ANSI cleanup intentionally matches ESC and CSI control characters.
// eslint-disable-next-line no-control-regex
const ANSI_SEQUENCE = /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g;

export function stripAnsi(value: string): string {
  return value.replace(ANSI_SEQUENCE, "");
}
