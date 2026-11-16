// Parity gate: builds every scenario in Road Builder and in Cartographer (headless VectorCraft, macOS) and compares the exported SVGs.
//   node scripts/parity.mjs <vectorcraft-cli> <road_builder_dir> [--png]
// With --png the Cartographer result is also exported as PNG into target/parity/ for a look. Exit 1 if any scenario differs or fails.
import "./lib/require-node.mjs";
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { headless } from "./lib/headless.mjs";
import { compareSvg } from "./lib/svgcompare.mjs";

const [cli, rbDir, ...flags] = process.argv.slice(2);
if (!cli || !rbDir) {
  console.error("usage: parity.mjs <vectorcraft-cli> <road_builder_dir> [--png]");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
const out = join(root, "target/parity");
mkdirSync(join(out, "plugins"), { recursive: true });
const OLD = { pid: "com.cosmicceo.roadbuilder.build", wasm: join(resolve(rbDir), "target/wasm32-unknown-unknown/release/road_builder_filter.wasm") };
const NEW = { pid: "com.cosmicceo.cartographer.build", wasm: join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm") };
const scenarios = JSON.parse(readFileSync(join(root, "scripts/parity/scenarios.json"), "utf8"));

const h = await headless(cli, root);
for (const p of [OLD, NEW]) {
  const dest = join(out, "plugins", `${p.pid}.wasm`);
  copyFileSync(p.wasm, dest); // a session reads only inside its root
  await h.run("plugin.install", { path: dest, replace: true });
}

/** Builds one scenario with one plug-in and exports it; the lines are drawn, each step selects its lines and runs a command, then Build. */
async function build(sc, plugin, svgPath, pngPath) {
  await h.run("file.new", { width: sc.size[0], height: sc.size[1] });
  const ids = [];
  for (const l of sc.lines) {
    const d = await h.tool("draw_path", { points: l.points.map(([x, y]) => ({ x, y })), ...(l.closed ? { closed: true } : {}) });
    ids.push(d.id ?? d.ids?.[0]);
  }
  for (const st of sc.steps) {
    await h.run("select.set", { ids: st.select.map((i) => ids[i]) });
    const params = Object.fromEntries(Object.entries(st.params).map(([k, v]) => [k, typeof v === "object" ? JSON.stringify(v) : v]));
    await h.run(`plugin.${plugin.pid}.${st.command}`, { params });
  }
  await h.run("select.all");
  await h.run("plugin.run", { id: plugin.pid });
  await h.run("file.export", { path: svgPath, format: "svg" });
  if (pngPath) await h.run("file.export", { path: pngPath, format: "png", scale: 4 });
}

let failed = 0;
for (const sc of scenarios) {
  const files = { old: join(out, `${sc.name}.old.svg`), new: join(out, `${sc.name}.new.svg`) };
  let line;
  try {
    await build(sc, OLD, files.old);
    await build(sc, NEW, files.new, flags.includes("--png") ? join(out, `${sc.name}.png`) : null);
    const r = compareSvg(readFileSync(files.old, "utf8"), readFileSync(files.new, "utf8"));
    if (!r.same) failed++;
    line = r.summary;
  } catch (e) {
    failed++;
    line = `ERROR ${String(e.message).slice(0, 150)}`;
  }
  console.log(`${sc.name}: ${line}`);
}
h.close();
console.log(failed ? `${failed} of ${scenarios.length} scenarios differ or failed` : `all ${scenarios.length} scenarios identical`);
process.exit(failed ? 1 : 0);
