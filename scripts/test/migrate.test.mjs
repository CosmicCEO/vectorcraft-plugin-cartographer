import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";
import { gunzipSync, gzipSync } from "node:zlib";
import { test } from "node:test";
import { IDS, TRACK_SCALE, migrateBytes, migrateDocument } from "../lib/migrate.mjs";

let n = 0;
const anchor = (x, y, extra = {}) => ({ p: [x, y], ...extra });
const path = (name, anchors, closed = false) => ({ id: ++n, name, kind: { type: "path", path: { subpaths: [{ anchors, closed }] } } });
const layer = (name, ...children) => ({ id: ++n, name, kind: { type: "layer", children } });
const doc = ({ layers = [], data = {} } = {}) => ({ format: "vectorcraft", version: 4, document: { layers, unknown: { pluginData: data } } });
const names = (file) => {
  const out = [];
  const walk = (node) => {
    if (node.name) out.push(node.name);
    for (const c of node.kind?.children ?? []) walk(c);
  };
  file.document.layers.forEach(walk);
  return out;
};
const find = (file, name) => {
  let hit;
  const walk = (node) => {
    if (node.name === name) hit = node;
    for (const c of node.kind?.children ?? []) walk(c);
  };
  file.document.layers.forEach(walk);
  return hit;
};
const RB = { version: 1, segments: { a: { cross_section: "lane back | lane fwd", anchor: "auto" } }, notes: ["road note"] };
const TL = { tracks: { t1: { tracks: 2 } }, yards: { t2: { mains: 1, left: 2 } }, joints: { j1: { kind: "crossing", lines: ["t1", "t2"] } }, notes: ["track note"] };

test("a road builder record moves to cartographers id", () => {
  const input = doc({ data: { [IDS.roadbuilder]: RB } });
  const { file, report } = migrateDocument(input);
  const rec = file.document.unknown.pluginData[IDS.cartographer];
  assert.equal(Object.keys(rec)[0], "schema");
  assert.equal(rec.schema, 1);
  assert.deepEqual(rec.segments, RB.segments);
  assert.deepEqual(rec.notes, ["road note"]);
  assert.deepEqual(file.document.unknown.pluginData[IDS.roadbuilder], RB, "the old record stays");
  assert.ok(report.some((l) => /1 road/.test(l)), report.join("\n"));
});

test("a track layer record moves and its lines are scaled", () => {
  const line = path("track.t1", [anchor(0, 0, { out: [100, 0] }), anchor(1000, 500, { in: [900, 500] })]);
  const ring = path("track.t2", [anchor(10, 10), anchor(20, 10), anchor(20, 20)], true);
  const road = path("road.a", [anchor(0, 0), anchor(500, 0)]);
  const input = doc({ layers: [layer("Layer 1", road, layer("track t1 spine", line, layer("deep", ring)))], data: { [IDS.tracklayer]: TL } });
  const { file, report } = migrateDocument(input);
  const k = TRACK_SCALE;
  assert.ok(Math.abs(k - 0.0564444) < 1e-6);
  const a = find(file, "track.t1").kind.path.subpaths[0].anchors;
  assert.deepEqual(a[0].out, [100 * k, 0]);
  assert.deepEqual(a[1].p, [1000 * k, 500 * k]);
  assert.deepEqual(a[1].in, [900 * k, 500 * k]);
  const r = find(file, "track.t2").kind.path.subpaths[0];
  assert.equal(r.closed, true);
  assert.deepEqual(r.anchors[2].p, [20 * k, 20 * k]);
  assert.deepEqual(find(file, "road.a").kind.path.subpaths[0].anchors[1].p, [500, 0], "a road line is not scaled");
  const rec = file.document.unknown.pluginData[IDS.cartographer];
  assert.deepEqual([rec.tracks, rec.yards, rec.joints], [TL.tracks, TL.yards, TL.joints]);
  assert.ok(report.some((l) => /2 track lines.*scaled/.test(l)), report.join("\n"));
});

test("both records are joined and notes are concatenated", () => {
  const { file } = migrateDocument(doc({ data: { [IDS.roadbuilder]: RB, [IDS.tracklayer]: TL } }));
  const rec = file.document.unknown.pluginData[IDS.cartographer];
  assert.deepEqual(rec.notes, ["road note", "track note"]);
  assert.ok(rec.segments && rec.tracks && rec.yards && rec.joints);
  assert.equal(rec.version, 1);
});

test("unknown keys are kept and reported", () => {
  const { file, report } = migrateDocument(doc({ data: { [IDS.roadbuilder]: { ...RB, centerline_model: { s: [1, 2] } } } }));
  assert.deepEqual(file.document.unknown.pluginData[IDS.cartographer].centerline_model, { s: [1, 2] });
  assert.ok(report.some((l) => /centerline_model/.test(l) && /Kept/.test(l)), report.join("\n"));
});

test("generated objects are dropped and counted", () => {
  const input = doc({ layers: [layer("road r1 drawing", path("gen.road.a.1.line.0", [anchor(0, 0), anchor(1, 1)]), layer("track t1 drawing", path("gen.track.t1.rails", [anchor(0, 0), anchor(1, 1)]))), layer("L", path("road.a", [anchor(0, 0), anchor(5, 5)]))], data: { [IDS.roadbuilder]: RB } });
  const { file, report } = migrateDocument(input);
  assert.deepEqual(names(file).filter((x) => x.startsWith("gen.")), []);
  assert.ok(names(file).includes("road.a"), "the line you drew stays");
  assert.ok(report.some((l) => /2 generated objects/.test(l) && /Build/.test(l)), report.join("\n"));
});

test("a document with nothing to migrate says so", () => {
  const input = doc({ data: { somebody_else: { x: 1 } } });
  const { file, report } = migrateDocument(input);
  assert.deepEqual(file, input);
  assert.match(report[0], /nothing to migrate/i);
});

test("a document that already has cartographer data is refused", () => {
  assert.throws(() => migrateDocument(doc({ data: { [IDS.cartographer]: { schema: 1 }, [IDS.roadbuilder]: RB } })), /already has Cartographer data/);
});

test("a newer old record version is reported", () => {
  const { report } = migrateDocument(doc({ data: { [IDS.roadbuilder]: { ...RB, version: 7 } } }));
  assert.ok(report.some((l) => /newer/.test(l)), report.join("\n"));
});

test("the input object is not changed", () => {
  const input = doc({ layers: [layer("L", path("track.t1", [anchor(100, 100)]), path("gen.x", [anchor(1, 1)]))], data: { [IDS.tracklayer]: TL } });
  const before = JSON.stringify(input);
  migrateDocument(input);
  assert.equal(JSON.stringify(input), before);
});

const bytesOf = (file, tail = Buffer.alloc(0)) => Buffer.concat([Buffer.from(JSON.stringify(file, null, 2), "latin1"), tail]);

test("bytes after the json survive", () => {
  const junk = randomBytes(100);
  const tail = Buffer.concat([Buffer.from("\u0000VCBLOBS", "latin1"), junk]);
  const input = bytesOf(doc({ data: { [IDS.roadbuilder]: RB } }), tail);
  const { bytes } = migrateBytes(input);
  assert.deepEqual(bytes.subarray(bytes.length - tail.length), tail);
  const text = bytes.toString("latin1");
  const json = JSON.parse(text.slice(0, text.indexOf("\u0000VCBLOBS")));
  assert.ok(json.document.unknown.pluginData[IDS.cartographer]);
});

test("a gzip file comes back gzip", () => {
  const input = gzipSync(bytesOf(doc({ data: { [IDS.roadbuilder]: RB } })));
  const { bytes } = migrateBytes(input);
  assert.equal(bytes[0], 0x1f);
  assert.ok(JSON.parse(gunzipSync(bytes).toString("latin1")).document.unknown.pluginData[IDS.cartographer]);
});

test("a file that is not a document says so plainly", () => {
  assert.throws(() => migrateBytes(Buffer.from("not json at all")), /not a VectorCraft document/);
});

// ---- the command

import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const SCRIPT = resolve(import.meta.dirname, "../migrate.mjs");
const run = (...args) => spawnSync(process.execPath, [SCRIPT, ...args], { encoding: "utf8" });
const sandbox = (file = doc({ data: { [IDS.roadbuilder]: RB } })) => {
  const dir = mkdtempSync(join(tmpdir(), "migrate-"));
  const input = join(dir, "in.vectorcraft");
  writeFileSync(input, bytesOf(file));
  return { dir, input, output: join(dir, "out.vectorcraft") };
};

test("the command writes a copy and leaves the input alone", () => {
  const { input, output } = sandbox();
  const before = readFileSync(input);
  const r = run(input, output);
  assert.equal(r.status, 0, r.stderr);
  assert.deepEqual(readFileSync(input), before, "the input is byte-identical");
  assert.ok(JSON.parse(readFileSync(output, "latin1")).document.unknown.pluginData[IDS.cartographer]);
  assert.match(r.stdout, /Road Builder: 1 road/);
  assert.match(r.stdout, /wrote/i);
});

test("it refuses to overwrite and says so", () => {
  const { input, output } = sandbox();
  writeFileSync(output, "precious");
  const r = run(input, output);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /already exists/);
  assert.equal(readFileSync(output, "utf8"), "precious");
});

test("it refuses the input as the output", () => {
  const { input } = sandbox();
  const r = run(input, input);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /different file/);
});

test("a missing input is a plain message", () => {
  const r = run(join(tmpdir(), "no-such-file.vectorcraft"), join(tmpdir(), "x.vectorcraft"));
  assert.equal(r.status, 1);
  assert.match(r.stderr, /cannot read/i);
});

test("nothing to migrate writes nothing", () => {
  const { input, output } = sandbox(doc({ data: { somebody: {} } }));
  const r = run(input, output);
  assert.equal(r.status, 0, r.stderr);
  assert.match(r.stdout, /nothing to migrate/i);
  assert.equal(existsSync(output), false);
});

test("a document that was migrated already is refused by the command too", () => {
  const { input, output } = sandbox(doc({ data: { [IDS.cartographer]: { schema: 1 }, [IDS.roadbuilder]: RB } }));
  const r = run(input, output);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /already has Cartographer data/);
  assert.equal(existsSync(output), false);
});

test("force replaces an existing output", () => {
  const { input, output } = sandbox();
  writeFileSync(output, "old");
  const r = run(input, output, "--force");
  assert.equal(r.status, 0, r.stderr);
  assert.ok(JSON.parse(readFileSync(output, "latin1")).document);
});

test("no arguments print the usage", () => {
  const r = run();
  assert.equal(r.status, 2);
  assert.match(r.stderr, /usage/i);
});

// ---- review fixes

import { linkSync } from "node:fs";

test("a document in the older centreline model is refused, not stripped of its drawing", () => {
  const input = doc({ layers: [layer("L", path("centerline", [anchor(0, 0), anchor(100, 0)]), path("gen.line.nbEdge", [anchor(0, 1), anchor(100, 1)]))], data: { [IDS.roadbuilder]: RB } });
  assert.throws(() => migrateDocument(input), /older centreline model/);
  const { input: file, output } = sandbox(input);
  const r = run(file, output);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /older centreline model/);
  assert.equal(existsSync(output), false);
});

test("the same file under another name is refused even with force", () => {
  const { dir, input } = sandbox();
  const before = readFileSync(input);
  const alias = join(dir, "hard.vectorcraft");
  linkSync(input, alias);
  const r = run(input, alias, "--force");
  assert.equal(r.status, 1);
  assert.match(r.stderr, /different file/);
  assert.deepEqual(readFileSync(input), before, "the input is byte-identical");
});

test("text written as a \\u escape comes back as valid UTF-8, and raw UTF-8 comes back byte for byte", () => {
  const escaped = Buffer.from(`${JSON.stringify(doc({ data: { [IDS.roadbuilder]: RB } }), null, 2).replace(/"version": 4,/, '"version": 4,\n  "title": "caf\\u00e9 \\u540d",')}`, "utf8");
  const { bytes } = migrateBytes(escaped);
  const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  assert.match(text, /"title": "café 名"/);
  const raw = Buffer.from(JSON.stringify(doc({ layers: [layer("Ünï } 名前 🚗", path("road.a", [anchor(0, 0), anchor(1, 1)]))], data: { [IDS.roadbuilder]: RB } }), null, 2), "utf8");
  const out = migrateBytes(raw).bytes;
  assert.match(new TextDecoder("utf-8", { fatal: true }).decode(out), /"name": "Ünï } 名前 🚗"/);
});
