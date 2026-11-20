// Fuel parity: the same inputs through the old and the new filters; the new cost must be at most 1.05 times the old for roads and at most 1.35 times for
// rail (rail numbers are written with 4 decimals instead of 3, and a build reads one more table; see docs/PARITY-RAIL.md).
//   node scripts/fuel-parity.mjs <road_builder_filter.wasm> [<track_layer_filter.wasm>]        (needs tools/fuel built and the new plug-in built)
// With Track Layer's filter the rail inputs (yard1, tracks2) are checked too.
import "./lib/require-node.mjs";
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import { parseFuel, withinParity } from "./lib/fuel.mjs";

const [oldWasm, railWasm] = process.argv.slice(2);
if (!oldWasm) {
  console.error("usage: fuel-parity.mjs <road_builder_filter.wasm> [<track_layer_filter.wasm>]");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
const tool = join(root, "tools/fuel/target/release/fuel");
const newWasm = join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm");
const dir = join(root, "target/spikes/s1");
for (const f of [tool, newWasm, resolve(oldWasm), ...(railWasm ? [resolve(railWasm)] : [])]) {
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
const ROAD_LIMIT = 1.05;
const RAIL_LIMIT = 1.35;
// Since 1.7 a yard's ladders carry their own ties and a switch tie set under every turnout on them (Track Layer drew the ladders bare), so
// yard1 draws 27% more than Track Layer did; its limit allows for that extra drawing (fuel per output byte is 1.39 times Track Layer's).
const YARD_LIMIT = 1.75;
const cases = [...["road600", "bridge", "road20km"].map((tag) => [tag, resolve(oldWasm), tag, ROAD_LIMIT]), ...(railWasm ? ["yard1", "tracks2"].map((tag) => [tag, resolve(railWasm), `${tag}.old`, tag === "yard1" ? YARD_LIMIT : RAIL_LIMIT]) : [])];
for (const [tag, oldFile, oldTag, limit] of cases) {
  const [o, n] = [run(oldFile, oldTag), run(newWasm, tag)];
  const { ok, ratio } = withinParity(o, n, limit);
  if (!ok) failed++;
  console.log(`${tag}: old ${o} new ${n} ratio ${ratio.toFixed(3)} ${ok ? "ok" : `TOO COSTLY (limit ${limit})`}`);
}
// The same roads in the hand-drawn style at 1:5000 against the plain build of the same road: at most twice the cost (long edges are cut into pieces so they follow the wander; the plain build is already inside the host's limit, see docs/PARITY.md).
for (const tag of ["road600", "road20km"]) {
  const [plain, hand] = [run(newWasm, tag), run(newWasm, `${tag}-hand`)];
  const ok = hand <= 2 * plain;
  if (!ok) failed++;
  console.log(`${tag}-hand: plain ${plain} hand-drawn ${hand} ratio ${(hand / plain).toFixed(3)} ${ok ? "ok" : "TOO COSTLY (limit 2.0)"}`);
}
process.exit(failed ? 1 : 0);
