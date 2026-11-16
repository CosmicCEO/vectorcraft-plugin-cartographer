// Fuel parity: the same inputs through Road Builder's and Cartographer's filters; the new cost must be at most 1.05 times the old.
//   node scripts/fuel-parity.mjs <road_builder_filter.wasm>        (needs tools/fuel built and the new plug-in built)
import "./lib/require-node.mjs";
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { parseFuel, withinParity } from "./lib/fuel.mjs";

const [oldWasm] = process.argv.slice(2);
if (!oldWasm) {
  console.error("usage: fuel-parity.mjs <road_builder_filter.wasm>");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
const tool = join(root, "tools/fuel/target/release/fuel");
const newWasm = join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm");
const dir = join(root, "target/spikes/s1");
for (const f of [tool, newWasm, resolve(oldWasm)]) {
  if (!existsSync(f)) {
    console.error(`fix: ${f} is missing. Build tools/fuel (cargo +stable build --release --manifest-path tools/fuel/Cargo.toml), run node scripts/build.mjs, and pass the old filter.`);
    process.exit(1);
  }
}
spawnSync(process.execPath, [join(root, "scripts/spikes/fuel-inputs.mjs"), dir], { stdio: "inherit" });

const run = (wasm, tag) => {
  const r = spawnSync(tool, [wasm, join(dir, `${tag}.in.json`), join(dir, `${tag}.params.json`)], { encoding: "utf8" });
  return parseFuel(r.stdout);
};
let failed = 0;
for (const tag of ["road600", "bridge", "road20km"]) {
  const [o, n] = [run(resolve(oldWasm), tag), run(newWasm, tag)];
  const { ok, ratio } = withinParity(o, n, 1.05);
  if (!ok) failed++;
  console.log(`${tag}: old ${o} new ${n} ratio ${ratio.toFixed(3)} ${ok ? "ok" : "TOO COSTLY"}`);
}
process.exit(failed ? 1 : 0);
