// Converting a saved VectorCraft document made with Road Builder and/or Track Layer into one Cartographer builds.
// Pure functions: `migrateDocument` works on the parsed file, `migrateBytes` on the bytes of a .vectorcraft file. See docs/MIGRATION.md.
import { gunzipSync, gzipSync } from "node:zlib";

export const IDS = {
  roadbuilder: "com.cosmicceo.roadbuilder.build",
  tracklayer: "com.cosmicceo.tracklayer.build",
  cartographer: "com.cosmicceo.cartographer.build",
};

/** Track Layer drew 72/25.4 document points per millimetre of model; Cartographer draws 0.16. */
export const TRACK_SCALE = 0.16 / (72 / 25.4);

/** The newest record version this tool knows how to read. */
const KNOWN_VERSION = 1;

const KNOWN_KEYS = new Set(["version", "schema", "settings", "notes", "segments", "junctions", "crossings", "hubs", "waters", "wets", "embankments", "transitions", "library", "tracks", "yards", "joints"]);

const COLLECTIONS = {
  segments: ["road", "roads"],
  junctions: ["junction", "junctions"],
  crossings: ["intersection", "intersections"],
  hubs: ["hub", "hubs"],
  waters: ["water feature", "water features"],
  wets: ["water crossing", "water crossings"],
  embankments: ["embankment", "embankments"],
  tracks: ["track", "tracks"],
  yards: ["yard", "yards"],
  joints: ["joint", "joints"],
};

const plural = (n, [one, many]) => `${n} ${n === 1 ? one : many}`;

/** "2 roads, 1 junction" for the collections of a record that are not empty. */
function counts(record) {
  return Object.entries(COLLECTIONS)
    .map(([key, words]) => [Object.keys(record[key] ?? {}).length, words])
    .filter(([n]) => n > 0)
    .map(([n, words]) => plural(n, words))
    .join(", ");
}

/** A number read losslessly (see `migrateBytes`) as a plain number. */
const num = (v) => (JSON.isRawJSON(v) ? Number(v.rawJSON) : v);

/** A deep copy that keeps losslessly read numbers as they are. */
const clone = (v) => (Array.isArray(v) ? v.map(clone) : v && typeof v === "object" && !JSON.isRawJSON(v) ? Object.fromEntries(Object.entries(v).map(([k, x]) => [k, clone(x)])) : v);

/** Scales every point and handle of a path in place. */
function scalePath(path, k) {
  for (const sub of path?.subpaths ?? []) {
    for (const a of sub.anchors ?? []) {
      for (const key of ["p", "in", "out"]) if (Array.isArray(a[key])) a[key] = a[key].map((v) => num(v) * k);
    }
  }
}

/** Scales a path, or every piece of a compound path (its pieces are unnamed children). */
function scaleNode(node, k) {
  if (node.kind?.path) scalePath(node.kind.path, k);
  for (const child of node.kind?.children ?? []) scaleNode(child, k);
}

/** Does this node, or anything inside it, have a name that is not a generated one? */
const holdsUserObjects = (node) => (node.kind?.children ?? []).some((c) => (typeof c.name === "string" && c.name !== "" && !c.name.startsWith("gen.")) || holdsUserObjects(c));

/**
 * Converts a parsed document. Returns `{file, report}`: a converted copy (the input is not changed) and plain-words lines saying what was done.
 * A document with nothing to convert comes back equal with a report that says so. Throws when it already has Cartographer data.
 */
export function migrateDocument(input) {
  const file = clone(input);
  const data = file.document?.unknown?.pluginData ?? {};
  if (data[IDS.cartographer]) throw new Error("This document already has Cartographer data. Migrate a copy of the original file, not one that was migrated already.");
  const [rb, tl] = [data[IDS.roadbuilder], data[IDS.tracklayer]];
  if (!rb && !tl) return { file, report: ["Nothing to migrate: this document has no Road Builder or Track Layer data."] };

  // Road Builder's older centreline model draws `gen.line.*` and `gen.stub.*` from a `centerline` object; Cartographer cannot redraw those, so removing
  // them would delete the document's only drawing. Refuse instead.
  const hasCenterline = (nodes) => nodes.some((n) => n?.name === "centerline" || hasCenterline(n?.kind?.children ?? []));
  if (hasCenterline(file.document.layers ?? [])) throw new Error("This document uses Road Builder's older centreline model (it has a `centerline` line), which Cartographer does not draw. Migrating it would remove its drawing, so nothing was changed: keep using Road Builder for it.");

  const report = [];
  const record = { schema: 1, version: 1 };
  const notes = [];
  for (const [label, old] of [["Road Builder", rb], ["Track Layer", tl]]) {
    if (!old) continue;
    const found = counts(old);
    report.push(`${label}: ${found || "no features stored"}.`);
    if (typeof num(old.version) === "number" && num(old.version) > KNOWN_VERSION) report.push(`${label} record version ${JSON.isRawJSON(old.version) ? old.version.rawJSON : old.version} is newer than this tool knows (${KNOWN_VERSION}); it was copied as it is, check the result.`);
    const unknown = Object.keys(old).filter((key) => !KNOWN_KEYS.has(key));
    if (unknown.length) report.push(`Kept but not used by Cartographer (from ${label}): ${unknown.join(", ")}.`);
    for (const [key, value] of Object.entries(old)) {
      if (key === "schema" || key === "version") continue;
      if (key === "notes") notes.push(...(Array.isArray(value) ? value : []));
      else record[key] = clone(value);
    }
  }
  if (rb && tl) {
    const shared = Object.keys(rb).filter((k) => k in tl && !["notes", "version", "schema"].includes(k));
    if (shared.length) report.push(`Found in both records: ${shared.join(", ")}; Track Layer's was kept. Check the result.`);
    report.push("This document has both Road Builder and Track Layer data: the track lines are scaled about the page origin and the roads are not, so their positions relative to each other change. Check the result.");
  }
  record.notes = notes;
  data[IDS.cartographer] = record;

  // Walk the layers: scale Track Layer's lines, drop what the old plug-ins generated (the first Build draws it again).
  let scaled = 0;
  const dropped = [];
  const walk = (nodes) => {
    for (let i = nodes.length - 1; i >= 0; i--) {
      const node = nodes[i];
      const name = typeof node?.name === "string" ? node.name : "";
      if (name.startsWith("gen.") && !holdsUserObjects(node)) {
        nodes.splice(i, 1);
        dropped.push(name);
        continue;
      }
      if (tl && /^track\..+/.test(name)) {
        scaleNode(node, TRACK_SCALE);
        scaled++;
      }
      if (Array.isArray(node?.kind?.children)) walk(node.kind.children);
    }
  };
  walk(file.document.layers ?? []);
  if (scaled) report.push(`${plural(scaled, ["track line", "track lines"])} scaled to Cartographer's 0.16 points per millimetre (factor ${TRACK_SCALE.toFixed(7)}).`);
  if (dropped.length) {
    report.push(`Removed ${plural(dropped.length, ["generated object", "generated objects"])}; select all and press Build in VectorCraft to draw them again.`);
    const shown = dropped.slice(0, 8).join(", ");
    report.push(`Removed by name: ${shown}${dropped.length > 8 ? ` and ${dropped.length - 8} more` : ""}.`);
  }
  report.push("The old plug-in records were left in the file.");
  return { file, report };
}

const MARKER = Buffer.from("\u0000VCBLOBS", "latin1");

/** Converts the bytes of a .vectorcraft file (plain or gzip JSON, possibly followed by a binary tail). Returns `{bytes, report}`. */
export function migrateBytes(bytes) {
  const zipped = bytes[0] === 0x1f && bytes[1] === 0x8b;
  const raw = zipped ? gunzipSync(bytes) : bytes;
  // The JSON part ends at the last `}` before the binary tail (a UTF-8 multibyte character never contains that byte). Work in bytes, so the tail is
  // kept exactly and the JSON is read and written as UTF-8.
  const marker = raw.indexOf(MARKER);
  const end = (marker >= 0 ? raw.lastIndexOf(0x7d, marker) : raw.lastIndexOf(0x7d)) + 1;
  let file;
  try {
    // Numbers that a plain parse would change (beyond 2^53, or written like 1.0) are kept as their own text.
    file = JSON.parse(raw.subarray(0, end).toString("utf8"), (k, v, ctx) => (typeof v === "number" && String(v) !== ctx?.source ? JSON.rawJSON(ctx.source) : v));
  } catch {
    throw new Error("This is not a VectorCraft document (its JSON cannot be read).");
  }
  if (!file?.document || typeof file.document !== "object") throw new Error("This is not a VectorCraft document (it has no document part).");
  const { file: converted, report } = migrateDocument(file);
  const out = Buffer.concat([Buffer.from(JSON.stringify(converted, null, 2), "utf8"), raw.subarray(end)]);
  return { bytes: zipped ? gzipSync(out) : out, report };
}
