// Every pairing of road, track and water (with itself and with each other), each drawn with every shape tool, is described, paired with its
// command and built in a headless VectorCraft session (never your open document). Prints one line per pairing and every failure.
//   node scripts/shape-matrix.mjs <vectorcraft-cli> [only: road+water ...]
import "./lib/require-node.mjs";
import { resolve } from "node:path";
import { headless } from "./lib/headless.mjs";
const [cli, only] = process.argv.slice(2);
const root = resolve(import.meta.dirname, "..");
const h = await headless(cli, root);
const ID = "com.cosmicceo.cartographer.build";
await h.run("plugin.install", { path: root + "/target/wasm32-unknown-unknown/release/cartographer_filter.wasm", replace: true });
const idOf = (r) => r?.id ?? r?.ids?.[0];
// Shape A lies across, B up and down (closed ones offset so their outlines cross).
const SHAPES = {
  pen: [async () => idOf(await h.tool("draw_path", { points: [{ x: 40, y: 150 }, { x: 260, y: 150 }] })), async () => idOf(await h.tool("draw_path", { points: [{ x: 150, y: 40 }, { x: 150, y: 260 }] }))],
  line: [() => h.run("shape.line", { x1: 40, y1: 150, x2: 260, y2: 150 }).then(idOf), () => h.run("shape.line", { x1: 150, y1: 40, x2: 150, y2: 260 }).then(idOf)],
  rect: [() => h.run("shape.rectangle", { x: 60, y: 110, width: 180, height: 80 }).then(idOf), () => h.run("shape.rectangle", { x: 110, y: 60, width: 80, height: 180 }).then(idOf)],
  rrect: [() => h.run("shape.rectangle", { x: 60, y: 110, width: 180, height: 80, radius: 20 }).then(idOf), () => h.run("shape.rectangle", { x: 110, y: 60, width: 80, height: 180, radius: 20 }).then(idOf)],
  ellipse: [() => h.run("shape.ellipse", { x: 60, y: 110, width: 180, height: 80 }).then(idOf), () => h.run("shape.ellipse", { x: 110, y: 60, width: 80, height: 180 }).then(idOf)],
  polygon: [() => h.run("shape.polygon", { cx: 150, cy: 150, radius: 90, sides: 6 }).then(idOf), () => h.run("shape.polygon", { cx: 190, cy: 180, radius: 90, sides: 6 }).then(idOf)],
  star: [() => h.run("shape.star", { cx: 150, cy: 150, radius1: 90, radius2: 50, points: 5 }).then(idOf), () => h.run("shape.star", { cx: 190, cy: 180, radius1: 90, radius2: 50, points: 5 }).then(idOf)],
  spiral: [() => h.run("shape.spiral", { cx: 150, cy: 150, radius: 90 }).then(idOf), () => h.run("shape.spiral", { cx: 190, cy: 180, radius: 90 }).then(idOf)],
  arc: [() => h.run("shape.arc", { x1: 40, y1: 200, x2: 260, y2: 100 }).then(idOf), () => h.run("shape.arc", { x1: 100, y1: 40, x2: 200, y2: 260 }).then(idOf)],
};
const CLOSED = new Set(["rect", "rrect", "ellipse", "polygon", "star"]);
const describe = {
  road: (id) => h.run(`plugin.${ID}.set_cross_section`, { params: { cross_section: "lane back | lane fwd" } }),
  track: (id) => h.run(`plugin.${ID}.set_tracks`, { params: { description: JSON.stringify({ tracks: 1 }) } }),
  water: (id, shape) => h.run(`plugin.${ID}.set_water`, { params: { description: JSON.stringify({ kind: CLOSED.has(shape) ? "still" : "flowing" }) } }),
};
const PAIR = { "road+road": ["set_crossing", {}], "road+track": ["set_level_crossing", { treatment: "level" }], "road+water": ["set_wet", { treatment: "bridge" }], "track+track": ["set_joint", { kind: "crossing" }], "track+water": null, "water+water": null };
const short = (e) => String(e?.message ?? e).replace(/^.*plug-in failed: /, "").replace(/^command_run: /, "").slice(0, 160);
const results = [];
for (const pair of (only ? [only] : Object.keys(PAIR))) {
  const [ea, eb] = pair.split("+");
  for (const sa of Object.keys(SHAPES)) for (const sb of Object.keys(SHAPES)) {
    const r = { pair, sa, sb };
    try {
      await h.run("file.new", { width: 300, height: 300 });
      const a = await SHAPES[sa][0](); await h.run("select.set", { ids: [a] });
      try { await describe[ea](a, sa); } catch (e) { r.describeA = short(e); }
      const b = await SHAPES[sb][1](); await h.run("select.set", { ids: [b] });
      try { await describe[eb](b, sb); } catch (e) { r.describeB = short(e); }
      if (PAIR[pair]) {
        await h.run("select.set", { ids: [a, b] });
        try { await h.run(`plugin.${ID}.${PAIR[pair][0]}`, { params: { description: JSON.stringify(PAIR[pair][1]) } }); r.paired = true; } catch (e) { r.pairErr = short(e); }
      }
      await h.run("select.all", {});
      try { await h.run("plugin.run", { id: ID }); r.built = true; } catch (e) { r.buildErr = short(e); }
    } catch (e) { r.setupErr = short(e); }
    results.push(r);
  }
}
h.close();
let failed = 0;
for (const pair of [...new Set(results.map((r) => r.pair))]) {
  const rs = results.filter((r) => r.pair === pair);
  const bad = rs.filter((r) => !r.built || (PAIR[pair] && !r.paired) || r.describeA || r.describeB || r.setupErr);
  failed += bad.length;
  console.log(`${pair}: ${rs.length - bad.length} of ${rs.length} shape combinations ${PAIR[pair] ? "pair and build" : "build (no pairing command)"}`);
  for (const r of bad) console.log(`  - ${r.sa} + ${r.sb}: ${r.setupErr ?? r.describeA ?? r.describeB ?? r.pairErr ?? r.buildErr}`);
}
process.exit(failed ? 1 : 0);
