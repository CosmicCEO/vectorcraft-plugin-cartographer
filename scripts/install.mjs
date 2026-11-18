// Installs the built plug-in into a running VectorCraft through its control port (no system plug-in folder is used).   node scripts/install.mjs
import "./lib/require-node.mjs";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { call } from "./lib/control.mjs";
import { wasmFile } from "./lib/locate.mjs";

// A source checkout has target/...; a release package has cartographer_filter.wasm next to the scripts folder.
const wasm = wasmFile(resolve(import.meta.dirname, ".."));
if (!wasm || !existsSync(wasm)) {
  console.error("fix: the plug-in is not built yet. Run: node scripts/build.mjs (a release package already contains it).");
  process.exit(1);
}
try {
  const r = await call("engine.execute", { command: "plugin.install", params: { path: wasm, replace: true } });
  console.log(`installed ${r.name} ${r.version}`);
} catch (e) {
  console.error(`fix: ${e.message}`);
  process.exit(1);
}
