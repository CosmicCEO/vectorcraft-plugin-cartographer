// Builds the plug-in (WebAssembly) and the native helper.   node scripts/build.mjs
import "./lib/require-node.mjs";
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { appleHelperPlan } from "./lib/applefm.mjs";
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

// The optional Apple Intelligence helper for the panel (macOS with swiftc only); never an error.
const swiftc = spawnSync("swiftc", ["--version"], { env, encoding: "utf8" }).status === 0;
const helper = appleHelperPlan(process.platform, swiftc);
if (helper) {
  const [cmd, ...args] = helper.build;
  const r = spawnSync(cmd, args, { cwd: root, env, stdio: "inherit" });
  if (r.status !== 0) console.error("note: the Apple Intelligence helper did not build (optional); the panel works without it.");
} else if (process.platform === "darwin") {
  console.log("note: swiftc not found; the Apple Intelligence helper was not built (optional).");
}
