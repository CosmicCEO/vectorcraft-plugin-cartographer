// Spike S3: what survives Save As, export and reopen? Uses Road Builder's plug-in only to make plug-in data and named shapes.
//   node scripts/spikes/s3-survival.mjs <vectorcraft-cli> <road_builder_filter.wasm>
// Prints a Markdown table. Works in a headless session, never in the user's open document.
import { copyFileSync, mkdirSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { gunzipSync, inflateSync } from "node:zlib";
import { headless } from "../lib/headless.mjs";

const [cli, filterWasm] = process.argv.slice(2);
if (!cli || !filterWasm) {
  console.error("usage: s3-survival.mjs <vectorcraft-cli> <road_builder_filter.wasm>");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "../..");
const out = join(root, "target/spikes/s3");
mkdirSync(out, { recursive: true });
const PID = "com.cosmicceo.roadbuilder.build";
const DRAWN = [[100, 300], [300, 300], [500, 250]];

const h = await headless(cli, root);
const notes = [];
const attempt = async (label, fn) => {
  try {
    return await fn();
  } catch (e) {
    notes.push(`${label}: ${String(e.message).slice(0, 200)}`);
    return null;
  }
};

// The session may only read inside `root`, so copy the plug-ins in first.
mkdirSync(join(out, "plugins"), { recursive: true });
for (const k of ["filter", "exporter", "importer"]) {
  const dest = join(out, "plugins", `road_builder_${k}.wasm`);
  copyFileSync(join(dirname(resolve(filterWasm)), `road_builder_${k}.wasm`), dest);
  await h.run("plugin.install", { path: dest, replace: true });
}
await h.run("file.new", { width: 600, height: 400 });
const drawn = await h.tool("draw_path", { points: DRAWN.map(([x, y]) => ({ x, y })) });
const id = drawn.id ?? drawn.ids?.[0];
await h.run("select.set", { ids: [id] });
await h.run(`plugin.${PID}.set_cross_section`, { params: { cross_section: "lane back | lane fwd" } });
await h.run("select.all");
await h.run("plugin.run", { id: PID });

const names = async () => ((await h.tool("doc_inspect", { kind: "Path", limit: 1000 })).nodes ?? []).map((n) => n.name);
const before = await names();

// Text forms of a file: as is, gunzipped, and every inflated PDF stream, so a string search can see inside.
const textsOf = (file) => {
  const raw = readFileSync(file);
  const out = [raw.toString("latin1")];
  try {
    out.push(gunzipSync(raw).toString("latin1"));
  } catch {}
  const s = raw.toString("latin1");
  for (const m of s.matchAll(/stream\r?\n/g)) {
    const end = s.indexOf("endstream", m.index);
    if (end < 0) continue;
    try {
      out.push(inflateSync(Buffer.from(s.slice(m.index + m[0].length, end), "latin1")).toString("latin1"));
    } catch {}
  }
  return out.join("\n");
};
// Direction: does the spine start at the first drawn point? Native copy: parse the JSON; SVG: the path data after the name.
const direction = (file, kind) => {
  const t = textsOf(file);
  const near = (a, b) => Math.abs(a - b) < 2;
  if (kind === "svg") {
    const m = /road\.[^"]*"[^>]*\sd="M\s*([\d.-]+)[ ,]+([\d.-]+)/.exec(t) || /\sd="M\s*([\d.-]+)[ ,]+([\d.-]+)[^"]*"[^>]*road\./.exec(t);
    return m ? (near(+m[1], DRAWN[0][0]) && near(+m[2], DRAWN[0][1]) ? "kept" : "reversed or moved") : "spine not found";
  }
  if (kind === "native") {
    const m = /"name":"road\.[^"]*"[\s\S]{0,4000}?"anchors":\[\{"p":\[([\d.-]+),([\d.-]+)\]/.exec(t.replace(/\s+/g, ""));
    return m ? (near(+m[1], DRAWN[0][0]) && near(+m[2], DRAWN[0][1]) ? "kept" : "reversed or moved") : "not found";
  }
  return "not checked";
};

const rows = [];
const check = (label, file, kind) => {
  const t = textsOf(file);
  rows.push({ label, data: /cross_section/.test(t) ? "yes" : "no", spine: /road\.r1|road\.main/.test(t) ? "yes" : "no", generated: /gen\.road/.test(t) ? "yes" : "no", direction: direction(file, kind) });
};

const native = join(out, "s3.vectorcraft");
if (await attempt("saveCopy", () => h.run("file.saveCopy", { path: native }))) check("Save copy (.vectorcraft)", native, "native");
for (const [ext, fmt] of [["svg", "svg"], ["pdf", "pdf"], ["pmap", "pmap"]]) {
  const p = join(out, `s3.${ext}`);
  if (await attempt(`export ${ext}`, () => h.run("file.export", { path: p, format: fmt }))) check(`Export ${ext.toUpperCase()}`, p, ext === "svg" ? "svg" : "other");
}

// Reopen the saved copy: what does the document hold now?
let reopened = null;
if (await attempt("reopen", () => h.tool("open_file", { path: native }))) {
  reopened = await names();
  const again = join(out, "s3-again.vectorcraft");
  if (await attempt("saveCopy again", () => h.run("file.saveCopy", { path: again }))) check("Reopen then Save copy", again, "native");
}
h.close();

console.log("| Artefact | Plug-in data kept | Our spine names | Generated shapes | Spine direction |\n|---|---|---|---|---|");
for (const r of rows) console.log(`| ${r.label} | ${r.data} | ${r.spine} | ${r.generated} | ${r.direction} |`);
console.log(`\nPaths before saving: ${before.length} (${before.slice(0, 4).join(", ")} ...)`);
if (reopened) console.log(`Paths after reopening the saved copy: ${reopened.length}`);
if (notes.length) console.log(`\nNotes:\n${notes.map((n) => `- ${n}`).join("\n")}`);
