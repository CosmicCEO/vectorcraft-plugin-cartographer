// Spike S1: writes <tag>.in.json and <tag>.params.json (for tools/fuel) for Road Builder scenarios, into <outdir>.
//   node scripts/spikes/fuel-inputs.mjs <outdir>
// Then: tools/fuel/target/release/fuel <road_builder_filter.wasm> <outdir>/<tag>.in.json <outdir>/<tag>.params.json
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const outdir = process.argv[2];
if (!outdir) {
  console.error("usage: fuel-inputs.mjs <outdir>");
  process.exit(2);
}
mkdirSync(outdir, { recursive: true });
const anchors = (pts) => pts.map(([x, y]) => ({ p: [x, y] }));
const line = (id, name, pts) => ({ id, name, path: { subpaths: [{ anchors: anchors(pts), closed: false }] } });
const TWO = "lane back | lane fwd";
const write = (tag, objects, documentData) => {
  writeFileSync(join(outdir, `${tag}.in.json`), JSON.stringify({ objects }));
  writeFileSync(join(outdir, `${tag}.params.json`), JSON.stringify({ _context: { mode: "filter", artboard: [0, 0, 800, 800], documentData } }));
};

// 1. A 600 m two-lane road, straight.
write("road600", [line(1, "road.a", [[100, 700], [100, 100]])], { segments: { a: { cross_section: TWO } } });

// 2. The same road crossing a 10 m creek with a bridge.
write("bridge", [line(1, "road.a", [[0, 400], [600, 400]]), line(2, "water.c", [[300, 700], [300, 100]])], {
  segments: { a: { cross_section: TWO } },
  waters: { c: { kind: "flowing", width: 10, bank_left: 1.5, bank_right: 1.5, slope: 2 } },
  wets: { x1: { road: "a", water: "c", treatment: "bridge" } },
});

// 3. A winding 20 km road of 2,001 points (the cost per input byte and per point).
const pts = Array.from({ length: 2001 }, (_, i) => [i * 10, 400 + 150 * Math.sin(i / 40)]);
write("road20km", [line(1, "road.a", pts)], { segments: { a: { cross_section: TWO } } });

// 4. Rail: a yard (2 mains, 4 + 2 spurs) and a double track on 1.5 m of model, written for Track Layer (72/25.4 points per mm: `<tag>.old.*`) and for
// Cartographer (0.16 points per mm: `<tag>.*`).
for (const [tag, spec, key] of [["yard1", { mains: 2, left: 4, right: 2, ends: "merge" }, "yards"], ["tracks2", { tracks: 2 }, "tracks"]]) {
  for (const [file, k] of [[tag, 0.16], [`${tag}.old`, 72 / 25.4]]) {
    const pts = [[100 * k, 400 * k], [1600 * k, 400 * k]];
    const [inName, paramsName] = [`${file}.in.json`, `${file}.params.json`];
    writeFileSync(join(outdir, inName), JSON.stringify({ objects: [line(1, "track.t1", pts)] }));
    writeFileSync(join(outdir, paramsName), JSON.stringify({ _context: { mode: "filter", artboard: [0, 0, 2000 * k, 1000 * k], documentData: { [key]: { t1: spec } } } }));
  }
}
