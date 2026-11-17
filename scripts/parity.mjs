// Parity gate: builds every scenario in Road Builder and in Cartographer (headless VectorCraft, macOS) and compares the exported SVGs.
//   node scripts/parity.mjs <vectorcraft-cli> <road_builder_dir> [--png]
//   node scripts/parity.mjs <vectorcraft-cli> <track_layer_dir> --scenarios <file> --old-id <plugin id> --old-wasm <file> --factor <n> [--png]
// With --factor the scenarios are in model millimetres: the old plug-in gets 72/25.4 points per mm, Cartographer 0.16, and the two SVGs are compared
// after multiplying every length of the old one by the factor (0.16 / (72 / 25.4) = 0.0564444), within 0.001 point.
// With --png the Cartographer result is also exported as PNG into target/parity/ for a look. Exit 1 if any scenario differs or fails.
import "./lib/require-node.mjs";
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { headless } from "./lib/headless.mjs";
import { compareSvg, compareSvgScaled, skipCollapsedRings } from "./lib/svgcompare.mjs";

const [cli, rbDir, ...flags] = process.argv.slice(2);
if (!cli || !rbDir) {
  console.error("usage: parity.mjs <vectorcraft-cli> <old_plugin_dir> [--png] [--scenarios <file> --old-id <id> --old-wasm <file> --factor <n>]");
  process.exit(2);
}
const option = (name) => (flags.includes(name) ? flags[flags.indexOf(name) + 1] : undefined);
const factor = option("--factor") === undefined ? null : Number(option("--factor"));
if (factor !== null && !(factor > 0)) {
  console.error("fix: --factor must be a positive number");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
const out = join(root, "target/parity");
mkdirSync(join(out, "plugins"), { recursive: true });
const OLD = {
  pid: option("--old-id") ?? "com.cosmicceo.roadbuilder.build",
  wasm: option("--old-wasm") ? resolve(option("--old-wasm")) : join(resolve(rbDir), "target/wasm32-unknown-unknown/release/road_builder_filter.wasm"),
  unit: factor === null ? 1 : 72 / 25.4, // points per scenario unit (millimetres with --factor)
};
const NEW = { pid: "com.cosmicceo.cartographer.build", wasm: join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm"), unit: factor === null ? 1 : 0.16 };
const scenarios = JSON.parse(readFileSync(option("--scenarios") ? resolve(option("--scenarios")) : join(root, "scripts/parity/scenarios.json"), "utf8"));

const h = await headless(cli, root);
for (const p of [OLD, NEW]) {
  const dest = join(out, "plugins", `${p.pid}.wasm`);
  copyFileSync(p.wasm, dest); // a session reads only inside its root
  await h.run("plugin.install", { path: dest, replace: true });
}

/** Builds one scenario with one plug-in and exports it; the lines are drawn, each step selects its lines and runs a command, then Build. */
async function build(sc, plugin, svgPath, pngPath) {
  const u = plugin.unit;
  const at = (p) => (Array.isArray(p) ? { x: p[0] * u, y: p[1] * u } : { x: p.x * u, y: p.y * u, ...(p.in && { in: p.in.map((v) => v * u) }), ...(p.out && { out: p.out.map((v) => v * u) }) });
  await h.run("file.new", { width: sc.size[0] * u, height: sc.size[1] * u });
  const ids = [];
  for (const l of sc.lines) {
    const d = await h.tool("draw_path", { points: l.points.map(at), ...(l.closed ? { closed: true } : {}) });
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
    const [a, b] = [readFileSync(files.old, "utf8"), readFileSync(files.new, "utf8")];
    const skip = factor === null ? { old: a, new: b, skipped: 0 } : skipCollapsedRings(a, b, factor);
    const r = factor === null ? compareSvg(a, b) : compareSvgScaled(skip.old, skip.new, { factor, tolerance: 0.001 });
    if (r.same && skip.skipped) r.summary = `identical (${skip.skipped} ring(s) Track Layer 0.9.1 collapsed to a point are skipped; Cartographer draws them)`;
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
