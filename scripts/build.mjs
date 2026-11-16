// Builds the plug-in (WebAssembly) and the native helper.   node scripts/build.mjs
import "./lib/require-node.mjs";
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { problems } from "./lib/checks.mjs";
import { rustEnv } from "./lib/env.mjs";
import { probeRust } from "./lib/probe.mjs";

const root = resolve(import.meta.dirname, "..");
const env = rustEnv(process.platform, process.env, existsSync);
const missing = problems({ nodeVersion: process.version, ...probeRust(env), hasCli: true, controlReachable: true });
if (missing.length) {
  for (const m of missing) console.error(`fix: ${m}`);
  process.exit(1);
}
const steps = [
  ["cargo", ["+stable", "build", "--release", "--target", "wasm32-unknown-unknown", "--workspace"]],
  ["cargo", ["+stable", "build", "--release", "-p", "cartographer-cli"]],
];
for (const [cmd, args] of steps) {
  const r = spawnSync(cmd, args, { cwd: root, env, stdio: "inherit" });
  if (r.error?.code === "ENOENT") {
    console.error("fix: cargo was not found. Install rustup (https://rustup.rs), then run: node scripts/check.mjs");
    process.exit(1);
  }
  if (r.status !== 0) process.exit(r.status ?? 1);
}
