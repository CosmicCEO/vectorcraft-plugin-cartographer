// What the Rust toolchain on this machine can do (a real probe: it runs cargo and rustup).
import { spawnSync } from "node:child_process";

/** `{hasRustup, hasWasmTarget}` for the stable toolchain, run with `env` (see rustEnv). */
export function probeRust(env) {
  const run = (cmd, args) => spawnSync(cmd, args, { env, encoding: "utf8" });
  return {
    hasRustup: run("cargo", ["+stable", "--version"]).status === 0,
    hasWasmTarget: (run("rustup", ["target", "list", "--installed", "--toolchain", "stable"]).stdout ?? "").includes("wasm32-unknown-unknown"),
  };
}
