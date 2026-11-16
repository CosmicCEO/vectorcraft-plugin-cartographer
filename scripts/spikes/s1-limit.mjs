// Spike S1: does a winding road of N points build in a headless session, or does the host refuse it for instruction budget?
//   node scripts/spikes/s1-limit.mjs <vectorcraft-cli> <road_builder_filter.wasm> <N> [<N> ...]
import { copyFileSync, mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { headless } from "../lib/headless.mjs";

const [cli, wasm, ...counts] = process.argv.slice(2);
if (!cli || !wasm || !counts.length) {
  console.error("usage: s1-limit.mjs <vectorcraft-cli> <road_builder_filter.wasm> <N> [<N> ...]");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "../..");
const dir = join(root, "target/spikes/s1");
mkdirSync(dir, { recursive: true });
const copy = join(dir, "road_builder_filter.wasm");
copyFileSync(resolve(wasm), copy);
const PID = "com.cosmicceo.roadbuilder.build";
const h = await headless(cli, root);
await h.run("plugin.install", { path: copy, replace: true });
for (const n of counts.map(Number)) {
  await h.run("file.new", { width: 30000, height: 800 });
  const pts = Array.from({ length: n }, (_, i) => ({ x: i * 10, y: 400 + 150 * Math.sin(i / 40) }));
  const d = await h.tool("draw_path", { points: pts });
  await h.run("select.set", { ids: [d.id ?? d.ids?.[0]] });
  await h.run(`plugin.${PID}.set_cross_section`, { params: { cross_section: "lane back | lane fwd" } });
  await h.run("select.all");
  try {
    await h.run("plugin.run", { id: PID });
    console.log(`${n} points (${(n - 1) * 10} m): built`);
  } catch (e) {
    console.log(`${n} points (${(n - 1) * 10} m): REFUSED ${String(e.message).slice(0, 160)}`);
  }
}
h.close();
