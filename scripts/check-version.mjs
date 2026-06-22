import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const app = JSON.parse(await readFile(resolve(root, "package.json"), "utf8"));
const cargo = await readFile(resolve(root, "src-tauri/Cargo.toml"), "utf8");
const kit = (await readFile(resolve(root, "kit/.lmbrain/VERSION"), "utf8")).trim();
const liveKit = (await readFile(resolve(root, ".lmbrain/VERSION"), "utf8")).trim();
const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

if (!cargoVersion || app.version !== cargoVersion || app.version !== kit || app.version !== liveKit) {
  throw new Error(`Version mismatch: package.json=${app.version}, Cargo.toml=${cargoVersion ?? "missing"}, kit=${kit}, liveKit=${liveKit}`);
}

console.log(`LMBrain app and kit are aligned at v${app.version}.`);
