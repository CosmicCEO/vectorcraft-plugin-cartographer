// Installs the built plug-in into a running VectorCraft through its control port (no system plug-in folder is used).   node scripts/install.mjs
import "./lib/require-node.mjs";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { call } from "./lib/control.mjs";

const wasm = resolve(import.meta.dirname, "../target/wasm32-unknown-unknown/release/cartographer_filter.wasm");
if (!existsSync(wasm)) {
  console.error("fix: the plug-in is not built yet. Run: node scripts/build.mjs");
  process.exit(1);
}
try {
  const r = await call("engine.execute", { command: "plugin.install", params: { path: wasm, replace: true } });
  console.log(`installed ${r.name} ${r.version}`);
} catch (e) {
  console.error(`fix: ${e.message}`);
  process.exit(1);
}
