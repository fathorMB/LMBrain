import { copyFile, mkdir, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { execFileSync } from "node:child_process";

const root = resolve(import.meta.dirname, "..");
const profileArg = process.argv.find((arg) => arg.startsWith("--profile="));
const profile = profileArg?.split("=")[1] || "release";
const extension = process.platform === "win32" ? ".exe" : "";

const rustc = execFileSync("rustc", ["-Vv"], { encoding: "utf8" });
const targetTriple = rustc.match(/^host:\s*(\S+)/m)?.[1];
if (!targetTriple) {
  throw new Error("Could not determine Rust target triple from `rustc -Vv`.");
}

const source = resolve(root, "target", profile, `lmbrain-mcp${extension}`);
await stat(source).catch(() => {
  throw new Error(`Missing MCP binary: ${source}. Run cargo build --${profile} -p lmbrain-mcp first.`);
});

const destination = resolve(
  root,
  "src-tauri",
  "binaries",
  `lmbrain-mcp-${targetTriple}${extension}`,
);
await mkdir(dirname(destination), { recursive: true });
await copyFile(source, destination);

console.log(`Prepared sidecar: ${destination}`);
