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

/** Scales every point and handle of a path in place. */
function scalePath(path, k) {
  for (const sub of path?.subpaths ?? []) {
    for (const a of sub.anchors ?? []) {
      for (const key of ["p", "in", "out"]) if (Array.isArray(a[key])) a[key] = a[key].map((v) => v * k);
    }
  }
}

/**
 * Converts a parsed document. Returns `{file, report}`: a converted copy (the input is not changed) and plain-words lines saying what was done.
 * A document with nothing to convert comes back equal with a report that says so. Throws when it already has Cartographer data.
 */
export function migrateDocument(input) {
  const file = structuredClone(input);
  const data = file.document?.unknown?.pluginData ?? {};
  if (data[IDS.cartographer]) throw new Error("This document already has Cartographer data. Migrate a copy of the original file, not one that was migrated already.");
  const [rb, tl] = [data[IDS.roadbuilder], data[IDS.tracklayer]];
  if (!rb && !tl) return { file, report: ["Nothing to migrate: this document has no Road Builder or Track Layer data."] };

  const report = [];
  const record = { schema: 1, version: 1 };
  const notes = [];
  for (const [label, old] of [["Road Builder", rb], ["Track Layer", tl]]) {
    if (!old) continue;
    const found = counts(old);
    report.push(`${label}: ${found || "no features stored"}.`);
    if (typeof old.version === "number" && old.version > KNOWN_VERSION) report.push(`${label} record version ${old.version} is newer than this tool knows (${KNOWN_VERSION}); it was copied as it is, check the result.`);
    const unknown = Object.keys(old).filter((key) => !KNOWN_KEYS.has(key));
    if (unknown.length) report.push(`Kept but not used by Cartographer (from ${label}): ${unknown.join(", ")}.`);
    for (const [key, value] of Object.entries(old)) {
      if (key === "schema" || key === "version") continue;
      if (key === "notes") notes.push(...(Array.isArray(value) ? value : []));
      else record[key] = structuredClone(value);
    }
  }
  record.notes = notes;
  data[IDS.cartographer] = record;

  // Walk the layers: scale Track Layer's lines, drop what the old plug-ins generated (the first Build draws it again).
  let scaled = 0;
  let dropped = 0;
  const walk = (nodes) => {
    for (let i = nodes.length - 1; i >= 0; i--) {
      const node = nodes[i];
      if (typeof node?.name === "string" && node.name.startsWith("gen.")) {
        nodes.splice(i, 1);
        dropped++;
        continue;
      }
      if (tl && /^track\..+/.test(node?.name ?? "") && node.kind?.path) {
        scalePath(node.kind.path, TRACK_SCALE);
        scaled++;
      }
      if (Array.isArray(node?.kind?.children)) walk(node.kind.children);
    }
  };
  walk(file.document.layers ?? []);
  if (scaled) report.push(`${plural(scaled, ["track line", "track lines"])} scaled to Cartographer's 0.16 points per millimetre (factor ${TRACK_SCALE.toFixed(7)}).`);
  if (dropped) report.push(`Removed ${plural(dropped, ["generated object", "generated objects"])}; select all and press Build in VectorCraft to draw them again.`);
  report.push("The old plug-in records were left in the file.");
  return { file, report };
}

const MARKER = "\u0000VCBLOBS";

/** Converts the bytes of a .vectorcraft file (plain or gzip JSON, possibly followed by a binary tail). Returns `{bytes, report}`. */
export function migrateBytes(bytes) {
  const zipped = bytes[0] === 0x1f && bytes[1] === 0x8b;
  const raw = zipped ? gunzipSync(bytes) : bytes;
  const text = raw.toString("latin1");
  const marker = text.indexOf(MARKER);
  const end = (marker >= 0 ? text.lastIndexOf("}", marker) : text.lastIndexOf("}")) + 1;
  let file;
  try {
    file = JSON.parse(text.slice(0, end));
  } catch {
    throw new Error("This is not a VectorCraft document (its JSON cannot be read).");
  }
  if (!file?.document || typeof file.document !== "object") throw new Error("This is not a VectorCraft document (it has no document part).");
  const { file: converted, report } = migrateDocument(file);
  // The text was read as latin1 (one character per byte), so writing it back as latin1 gives the original bytes; a character above 255 can only
  // have come from a \u escape and is written as one again.
  const json = JSON.stringify(converted, null, 2).replace(/[^\u0000-ÿ]/g, (c) => `\\u${c.charCodeAt(0).toString(16).padStart(4, "0")}`);
  const out = Buffer.concat([Buffer.from(json, "latin1"), raw.subarray(Buffer.byteLength(text.slice(0, end), "latin1"))]);
  return { bytes: zipped ? gzipSync(out) : out, report };
}
