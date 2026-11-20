// Reading what the panel needs from the live document: the selection, road names, and the descriptions the plug-in keeps.
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { gunzipSync } from "node:zlib";

export const PLUGIN_ID = "com.cosmicceo.cartographer.build";
export const SET_SETTINGS = `plugin.${PLUGIN_ID}.set_settings`;
export const SET_CROSS_SECTION = `plugin.${PLUGIN_ID}.set_cross_section`;
export const SET_JUNCTION = `plugin.${PLUGIN_ID}.set_junction`;
export const SET_CROSSING = `plugin.${PLUGIN_ID}.set_crossing`;
export const SET_HUB = `plugin.${PLUGIN_ID}.set_hub`;
export const SET_TRANSITION = `plugin.${PLUGIN_ID}.set_transition`;
export const SET_WATER = `plugin.${PLUGIN_ID}.set_water`;
export const SET_EMBANKMENT = `plugin.${PLUGIN_ID}.set_embankment`;
export const SET_WET = `plugin.${PLUGIN_ID}.set_wet`;
export const SET_TRACKS = `plugin.${PLUGIN_ID}.set_tracks`;
export const SET_YARD = `plugin.${PLUGIN_ID}.set_yard`;
export const SET_JOINT = `plugin.${PLUGIN_ID}.set_joint`;
export const SET_LEVEL_CROSSING = `plugin.${PLUGIN_ID}.set_level_crossing`;

/** The id of a track line (`track.a` is `a`), or null. Mirrors core/src/rail_builder.rs. */
export function trackId(name = "") {
  return name.startsWith("track.") && name.length > 6 ? name.slice(6) : null;
}

/** The id of a water line (`water.c` is `c`, plain `water` is `main`), or null. Mirrors core/src/water.rs. */
export function waterId(name = "") {
  if (name === "water") return "main";
  if (!name.startsWith("water.")) return null;
  return name.slice(6).split(".")[0] || "main";
}

/** The segment id of a road line (`road.a` is `a`, plain `road` is `main`), or null. Mirrors core/src/roads.rs. */
export function segmentId(name = "") {
  if (name === "road") return "main";
  if (!name.startsWith("road.")) return null;
  return name.slice(5).split(".")[0] || "main";
}

/** The id of a junction line (`junction.x` is `x`, plain `junction` is `main`), or null. Mirrors core/src/junction.rs. */
export function junctionId(name = "") {
  if (name === "junction") return "main";
  if (!name.startsWith("junction.")) return null;
  return name.slice(9).split(".")[0] || "main";
}

/** Which road or junction a generated shape belongs to: `gen.road.<id>.<piece>.…` or `gen.junction.<id>.…`, else null. */
export function shapeOwner(name = "") {
  let m = /^gen\.road\.([^.]+)\.(\d+)\./.exec(name);
  if (m) return { role: "road-shape", segment: m[1], piece: Number(m[2]) };
  m = /^gen\.junction\.([^.]+)\./.exec(name);
  if (m) return { role: "junction-shape", segment: m[1] };
  m = /^gen\.crossing\.([^.]+)\./.exec(name);
  if (m) return { role: "crossing-shape", segment: m[1] };
  m = /^gen\.hub\.([^.]+)\./.exec(name);
  if (m) return { role: "hub-shape", segment: m[1] };
  m = /^gen\.water\.([^.]+)\./.exec(name);
  if (m) return { role: "water-shape", segment: m[1] };
  m = /^gen\.wet\.([^.]+)\./.exec(name);
  if (m) return { role: "wet-shape", segment: m[1] };
  m = /^gen\.track\.([^.]+)\./.exec(name);
  if (m) return { role: "track-shape", track: m[1] };
  return null;
}

const OWN = /^(gen\.|profile\.|flag\.|arrow\.)|^centerline$|^[A-Z]\d+\./;

/** What a selected object is to the panel: a road piece, a plain line that could become a road, or something else. */
export function classify(node) {
  const isPath = isLineKind(node.kind);
  const segment = isPath ? segmentId(node.name) : null;
  if (segment) return { role: "road", segment };
  const junction = isPath ? junctionId(node.name) : null;
  if (junction) return { role: "junction", segment: junction };
  const water = isPath ? waterId(node.name) : null;
  if (water) return { role: "water", segment: water };
  const track = isPath ? trackId(node.name) : null;
  if (track) return { role: "track", track };
  const shape = isPath ? shapeOwner(node.name) : null;
  if (shape) return shape;
  if (isPath && !OWN.test(node.name ?? "")) return { role: "line", segment: null };
  return { role: "other", segment: null };
}

export async function inspectState(control) {
  const r = await control.call("document.inspect", { depth: 0 });
  // `title` is the open file (two copies of one document differ there); `fileInfoTitle` is the name stored inside it (shared by copies).
  return { revision: r.revision, title: r.title ?? r.fileInfoTitle ?? "", selection: r.selection ?? [] };
}

export async function nodes(control, ids) {
  if (!ids.length) return [];
  const r = await control.call("document.inspect", { ids });
  return r.nodes ?? [];
}

/** Kinds that cannot be a road, track or water line. Every other kind VectorCraft reports is one: the pen, pencil, star, spiral and arc
 *  make `Path`s, and the shape tools keep their own kind (Line, Ellipse, Rectangle, Rounded Rectangle, Polygon, and any added later).
 *  Groups (also the grid and flare tools) are looked into, not taken whole. */
const NOT_LINES = new Set(["Layer", "Group", "Type", "Image", "Compound Path", "Symbol", "Mesh", "Graph"]);
export const isLineKind = (kind) => typeof kind === "string" && !NOT_LINES.has(kind);

/** Every line-like object in the document, in document order (the order the plug-in numbers a road's pieces in), each with `path`: the ids of
 *  the layers it sits in, outermost first. One walk of the layer tree. */
export async function lineNodes(control) {
  const r = await control.call("document.inspect", { depth: 16 });
  const out = [];
  const walk = (list, layers) => {
    for (const x of list ?? []) {
      if (x.kind === "Layer") walk(x.children, [...layers, x.id]);
      else if (isLineKind(x.kind)) out.push({ id: x.id, name: x.name ?? "", kind: x.kind, path: layers });
      else walk(x.children, layers); // a group: its members belong to the same layer
    }
  };
  walk(r.layers, []);
  return out;
}

/** All paths in the document (id and name), for counting the pieces of a road. */
export async function allPaths(control) {
  return (await lineNodes(control)).map((n) => ({ id: n.id, name: n.name ?? "" }));
}

/** The JSON part of a saved .vectorcraft file (a binary preview may follow it, after a NUL and VCBLOBS). */
export function parseNativeFile(bytes) {
  const buf = bytes[0] === 0x1f && bytes[1] === 0x8b ? gunzipSync(bytes) : bytes;
  const text = buf.toString("latin1");
  const marker = text.indexOf("\u0000VCBLOBS");
  const end = (marker >= 0 ? text.lastIndexOf("}", marker) : text.lastIndexOf("}")) + 1;
  return JSON.parse(text.slice(0, end));
}

/** The path of every track line in a saved document: {id: {subpaths: [...]}} (document points, with Bezier handles). */
export function trackLines(doc) {
  const lines = {};
  const walk = (node) => {
    if (node?.kind?.type === "path") {
      const id = trackId(node.name ?? "");
      if (id && node.kind.path) lines[id] = node.kind.path;
    }
    for (const child of node?.kind?.children ?? []) walk(child);
  };
  for (const layer of doc.layers ?? []) walk(layer);
  return lines;
}

/** The descriptions the plug-in keeps with the document: {segments, tracks, yards, joints, notes, ...} and `lines` (the geometry of each track
 *  line, which the panel needs to say how two lines meet), read from a saved copy. */
export async function readSpec(control) {
  const dir = mkdtempSync(join(tmpdir(), "cartographer-panel-"));
  const path = join(dir, "copy.vectorcraft");
  try {
    await control.run("file.saveCopy", { path });
    // The app may write the copy from a background thread: wait until it is there and has stopped growing.
    let last = -1;
    for (let i = 0; i < 60; i++) {
      const size = existsSync(path) ? statSync(path).size : 0;
      if (size > 0 && size === last) break;
      last = size;
      await new Promise((r) => setTimeout(r, 50));
    }
    const doc = parseNativeFile(readFileSync(path)).document ?? {};
    return { segments: {}, tracks: {}, yards: {}, joints: {}, level_crossings: {}, ...inMetres(doc.unknown?.pluginData?.[PLUGIN_ID] ?? {}), lines: trackLines(doc) };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

/** A record saved before schema 2 kept its railway in model millimetres (1:160): said in metres of ground here, as the plug-in reads it
 *  (core/src/rail_units.rs `upgrade`), so the panel never shows or writes back millimetres as metres. */
export function inMetres(data) {
  if ((data.schema ?? 0) >= 2) return data;
  const m = (x) => (typeof x === "number" ? Math.round((x / 6.25) * 1e6) / 1e6 : x);
  const rec = (r) => {
    if (!r || typeof r !== "object") return r;
    const out = { ...r };
    for (const k of ["spacing", "at", "length"]) if (k in out) out[k] = m(out[k]);
    if (Array.isArray(out.crossovers)) out.crossovers = out.crossovers.map(rec);
    return out;
  };
  const table = (t) => (t && typeof t === "object" ? Object.fromEntries(Object.entries(t).map(([k, v]) => [k, rec(v)])) : t);
  return { ...data, ...(data.tracks ? { tracks: table(data.tracks) } : {}), ...(data.yards ? { yards: table(data.yards) } : {}), schema: 2 };
}

/** The dash that turns a tie line into ties, in document points: a tie's width on, the rest of the pitch off (core/src/rail_standards.rs:
 *  TIE_WIDTH 1.4 mm and TIE_PITCH 3.1 mm of model, 0.16 points per mm). The plug-in writes each route's ties as one line as wide as a tie is
 *  long; VectorCraft ignores dashes in a plug-in's output, so the panel sets them after every build. */
export const TIE_DASH = [1.4 * 0.16, (3.1 - 1.4) * 0.16];

/** Dashes the tie lines in the document (`gen.track.*.ties`) that are not in `skip`, in one command. Returns their ids. */
export async function dashTies(control, skip = new Set()) {
  const ids = (await lineNodes(control)).filter((n) => /^gen\.track\..+\.ties$/.test(n.name) && !skip.has(n.id)).map((n) => n.id);
  if (ids.length) await control.run("stroke.set", { ids, dash: TIE_DASH, cap: "butt" });
  return ids;
}

