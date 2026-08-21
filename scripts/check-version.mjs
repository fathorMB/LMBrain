import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const app = JSON.parse(await readFile(resolve(root, "package.json"), "utf8"));
const tauriCargo = await readFile(resolve(root, "src-tauri/Cargo.toml"), "utf8");
const coreCargo = await readFile(resolve(root, "lmbrain-core/Cargo.toml"), "utf8");
const mcpCargo = await readFile(resolve(root, "lmbrain-mcp/Cargo.toml"), "utf8");
const kit = (await readFile(resolve(root, "kit/.lmbrain/VERSION"), "utf8")).trim();

const tauriVersion = tauriCargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const coreVersion = coreCargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const mcpVersion = mcpCargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (
  !tauriVersion ||
  !coreVersion ||
  !mcpVersion ||
  app.version !== tauriVersion ||
  app.version !== coreVersion ||
  app.version !== mcpVersion ||
  app.version !== kit
) {
  throw new Error(
    `Version mismatch: package.json=${app.version}, src-tauri=${tauriVersion ?? "missing"}, lmbrain-core=${coreVersion ?? "missing"}, lmbrain-mcp=${mcpVersion ?? "missing"}, kit=${kit}`
  );
}

console.log(`LMBrain workspace crates, app, and kit are aligned at v${app.version}.`);
