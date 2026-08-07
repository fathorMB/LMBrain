/**
 * Tauri exposes custom URI scheme protocols as `http://<scheme>.localhost/`
 * only on Windows (and Android); Linux WebKitGTK and macOS WKWebView load
 * them as `<scheme>://localhost/`. Using the wrong form silently renders a
 * blank frame (issue #97).
 *
 * Lives outside the component file so DesignView keeps exporting only
 * components (react-refresh/only-export-components).
 */
export function designPreviewUrl(
  entryPath: string,
  userAgent: string = navigator.userAgent
) {
  const normalized = entryPath.replace(/\\/g, "/").replace(/^\/+/, "");
  const encoded = normalized.split("/").map(encodeURIComponent).join("/");
  const isWindows = /windows|win32|win64/i.test(userAgent);
  return isWindows
    ? `http://lmbrain-design.localhost/${encoded}`
    : `lmbrain-design://localhost/${encoded}`;
}
