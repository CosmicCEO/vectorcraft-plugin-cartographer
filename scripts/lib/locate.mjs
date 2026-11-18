// Where the built files are: in a source checkout (target/...) or in a release package (next to the scripts).
import { existsSync } from "node:fs";
import { join } from "node:path";

const first = (candidates) => candidates.find((p) => existsSync(p)) ?? null;

/** The plug-in file: a fresh build in a checkout, else the one in a release package. `null` when neither exists. */
export function wasmFile(root) {
  return first([join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm"), join(root, "cartographer_filter.wasm")]);
}

/** The native helper: a fresh build in a checkout, else the one in a release package. `null` when neither exists. */
export function cliBinary(root) {
  return first([join(root, "target/release/cartographer-cli"), join(root, "bin/cartographer-cli")]);
}

/** Is this folder a release package (no source to build from)? */
export function isPackaged(root) {
  return !existsSync(join(root, "Cargo.toml"));
}
