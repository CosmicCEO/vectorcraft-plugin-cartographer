import assert from "node:assert/strict";
import { after, test } from "node:test";
import { Control } from "../lib/control.mjs";
import { PLUGIN_ID, SET_JOINT, SET_LEVEL_CROSSING, SET_TRACKS, SET_YARD } from "../lib/doc.mjs";
import { Panel } from "../lib/panel.mjs";
import { fakeVectorCraft } from "./fake-vectorcraft.mjs";

const YARD = '{"kind": "yard", "mains": 2, "left": 8, "right": 4, "ends": "merge"}';
const agent = (text) => ({ default: "fake", providers: { fake: { kind: "function", label: "Fake", fn: () => text } } });

// The commands the user's own steps made, without the layer tidying the panel does after a build.
const steps = (vc) => vc.doc.calls.filter((c) => c.command !== "file.saveCopy" && !c.command.startsWith("layer.")).map((c) => c.command);

const cleanups = [];
after(() => cleanups.forEach((f) => f()));

async function setup(vc, answer = YARD) {
  const control = new Control({ port: vc.port });
  const panel = new Panel({ control, providers: agent(answer) });
  const done = () => { control.close(); vc.close(); };
  cleanups.push(done); // even when a test fails half way, nothing is left open
  await panel.tick();
  return { panel, control, done };
}

// A plain line is a road until the user says it is track (the mode card); the panel then shows the track card and describes it as a track.
async function trackMode(panel) {
  panel.setMode("track");
  await panel.tick(true);
}

test("a plain selected line is offered for description", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }], selection: [5] });
  const { panel, done } = await setup(vc);
  assert.equal(panel.snapshot().target.kind, "line");
  assert.equal(panel.snapshot().track, null);
  done();
});

test("describe a yard in words, then Draw: the right commands, in order, and the selection comes back", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }, { id: 9, name: "gen.track.zz.m1.rails" }], selection: [5] });
  const { panel, done } = await setup(vc);
  await trackMode(panel);
  await panel.propose("two mainlines enter a yard with 8 spurs to the left and four to the right, they merge back down to two at the end", "fake");
  const s = panel.snapshot();
  assert.equal(s.pending.kind, "yard");
  assert.deepEqual([s.pending.spec.mains, s.pending.spec.left, s.pending.spec.right, s.pending.spec.ends], [2, 8, 4, "merge"]);
  assert.match(s.pending.summary, /2 mains, 8 spurs left, 4 spurs right, merging back/);
  assert.ok(s.pending.min_line_m > 400 && s.pending.min_line_m < 408, `needs about 404 m of line: ${s.pending.min_line_m}`);
  assert.deepEqual(s.pending.changes, ["new: a yard"]);
  assert.deepEqual(steps(vc), [], "nothing is applied until Draw");

  await panel.draw();
  assert.deepEqual(steps(vc), ["select.set", SET_YARD, "select.all", "plugin.run", "select.set"]);
  const names = vc.doc.layers.map((l) => l.name);
  assert.ok(names.includes("track t1") && names.includes("track t1 spine") && names.includes("track t1 drawing"), `each track gets a layer with a spine and a drawing: ${names}`);
  assert.deepEqual(vc.doc.paths.find((x) => x.id === 5).path, [vc.doc.layers.find((l) => l.name === "track t1 spine").id], "the line is on its spine layer");
  // What is sent is the program's own checked form of the description, with the defaults filled in.
  assert.deepEqual(JSON.parse(vc.doc.calls.find((c) => c.command === SET_YARD).params.params.description), { mains: 2, left: 8, right: 4, ends: "merge", frog: 6, spacing: 3.9624, at: 0, ballast: true, ties: true });
  assert.deepEqual(vc.doc.calls.find((c) => c.command === "plugin.run").params, { id: PLUGIN_ID });
  assert.deepEqual(vc.doc.selection, [5], "the user's selection is restored");
  const after = panel.snapshot();
  assert.equal(after.pending, null);
  assert.deepEqual([after.target.kind, after.track.kind, after.track.described], ["track", "yard", true]);
  assert.match(after.track.summary, /24 turnouts/);
  done();
});

test("parallel tracks use Describe Track, and changing a yard to tracks is said in words", async () => {
  const spec = { tracks: {}, yards: { t1: { mains: 1, left: 2, ends: "stub", frog: 6, spacing: 3.9624, at: 0, ballast: true, ties: true } } };
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2], spec });
  const { panel, done } = await setup(vc, '{"tracks": 3, "spacing": "14ft"}');
  assert.equal(panel.snapshot().track.kind, "yard", "a stored yard is shown as a yard");
  await panel.propose("make it three parallel tracks", "fake");
  const p = panel.snapshot().pending;
  assert.equal(p.kind, "tracks");
  assert.deepEqual(p.changes, ["changes from a yard to parallel tracks"]);
  await panel.draw();
  assert.ok(steps(vc).includes(SET_TRACKS) && !steps(vc).includes(SET_YARD));
  assert.equal(vc.doc.spec.tracks.t1.tracks, 3);
  assert.equal(vc.doc.spec.yards.t1, undefined, "the yard description is replaced");
  done();
});

test("an edit shows what changed, field by field", async () => {
  const stored = { mains: 2, left: 8, right: 4, ends: "merge", frog: 6, spacing: 3.9624, at: 0, ballast: true, ties: true };
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2], spec: { tracks: {}, yards: { t1: stored } } });
  const { panel, done } = await setup(vc, JSON.stringify({ kind: "yard", ...stored, left: 10 }));
  await panel.propose("two more spurs on the left", "fake");
  assert.deepEqual(panel.snapshot().pending.changes, ["spurs on the left: 8 → 10"]);
  done();
});

test("clicking built track selects its line", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }, { id: 7, name: "gen.track.t1.m1.rails" }], selection: [7] });
  const { panel, done } = await setup(vc);
  const s = panel.snapshot();
  assert.deepEqual([s.target.kind, s.target.id, s.target.ids], ["track", "t1", [2]]);
  done();
});

test("Draw with nothing proposed just rebuilds", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2] });
  const { panel, done } = await setup(vc);
  await panel.draw();
  assert.deepEqual(steps(vc), ["select.all", "plugin.run", "select.set"]);
  assert.equal(panel.snapshot().message, "Rebuilt.", "the result of Draw is still said after the panel reads the document again");
  done();
});

test("an invalid answer from the agent never reaches VectorCraft", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }], selection: [5] });
  const { panel, done } = await setup(vc, '{"kind": "yard", "left": 99}');
  await trackMode(panel);
  await panel.propose("a huge yard", "fake");
  assert.equal(panel.snapshot().pending, null);
  assert.match(panel.snapshot().message, /could not produce a valid description/);
  assert.deepEqual(steps(vc), []);
  done();
});

test("the plug-in's refusal (line too short) is shown, not hidden", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }], selection: [5], rejectYard: "This yard needs 2524 mm of line from 0 mm on, and the selected line gives 500 mm." });
  const { panel, done } = await setup(vc);
  await trackMode(panel);
  await panel.propose("the big yard", "fake");
  await panel.draw();
  const s = panel.snapshot();
  assert.match(s.message, /Could not draw: .*needs 2524 mm/);
  assert.equal(s.busy, null);
  assert.ok(!steps(vc).includes("plugin.run"), "nothing was built");
  done();
});

test("with no document open the panel says so", async () => {
  const vc = await fakeVectorCraft({ hasDocument: false });
  const { panel, done } = await setup(vc);
  assert.match(panel.snapshot().error, /Open or create a document/);
  done();
});

test("a layer the user hid is shown for the build and hidden again afterwards", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2] });
  vc.doc.layers.push({ id: 2000, name: "track t1 spine", visible: false, locked: true, parent: null });
  const { panel, done } = await setup(vc);
  await panel.draw();
  const sets = vc.doc.calls.filter((c) => c.command === "layer.setProps" && c.params.id === 2000).map((c) => [c.params.visible, c.params.locked]);
  assert.deepEqual(sets, [[true, false], [false, true]]);
  done();
});

// ---- two lines that meet

// Document points: 0.16 per millimetre of model.
const path = (pts) => ({ subpaths: [{ anchors: pts.map(([x, y]) => ({ p: [x * 0.16, y * 0.16] })), closed: false }] });
const CROSS = [
  { id: 2, name: "track.a", geom: path([[0, 0], [600, 0]]) },
  { id: 3, name: "track.b", geom: path([[300, -200], [300, 200]]) },
];

test("two selected track lines are a joint, and the panel says how they meet", async () => {
  const vc = await fakeVectorCraft({ paths: CROSS, selection: [2, 3] });
  const { panel, done } = await setup(vc);
  const s = panel.snapshot();
  assert.deepEqual([s.target.kind, s.target.lines], ["joint", ["a", "b"]]);
  assert.equal(s.joint.found.ok, true);
  assert.equal(s.joint.found.kind, "crossing");
  assert.match(s.joint.found.summary, /lines a and b cross, 48 m along a and 32 m along b, at 90.0 degrees/);
  assert.equal(s.joint.described, false);
  done();
});

test("describe a crossing in words, then Draw: Join Two Lines, then the build", async () => {
  const vc = await fakeVectorCraft({ paths: CROSS, selection: [2, 3] });
  const { panel, done } = await setup(vc, '{"kind": "crossing", "main": "a"}');
  await panel.propose("they cross and a is the main", "fake");
  const p = panel.snapshot().pending;
  assert.deepEqual([p.kind, p.spec, p.summary, p.changes], ["joint", { kind: "crossing", main: "a" }, "a crossing, line a runs through", ["new: a crossing, line a runs through"]]);
  assert.equal(vc.doc.calls.filter((c) => c.command === SET_JOINT).length, 0, "not applied until Draw");
  await panel.draw();
  assert.deepEqual(steps(vc), ["select.set", SET_JOINT, "select.all", "plugin.run", "select.set"]);
  assert.deepEqual(vc.doc.selection, [2, 3], "the user's selection comes back");
  assert.deepEqual(vc.doc.spec.joints.j1, { kind: "crossing", main: "a", lines: ["a", "b"] });
  const s = panel.snapshot();
  assert.equal(s.pending, null);
  assert.deepEqual([s.joint.described, s.joint.summary], [true, "a crossing, line a runs through"]);
  done();
});

test("a stored joint is shown, and an edit says what changed", async () => {
  const vc = await fakeVectorCraft({ paths: CROSS, selection: [2, 3], spec: { tracks: {}, yards: {}, joints: { j1: { kind: "crossing", main: "a", lines: ["a", "b"] } } } });
  const { panel, done } = await setup(vc, '{"kind": "crossing", "main": "b"}');
  assert.equal(panel.snapshot().joint.summary, "a crossing, line a runs through");
  await panel.propose("b is the main", "fake");
  assert.deepEqual(panel.snapshot().pending.changes, ["from a crossing, line a runs through to a crossing, line b runs through"]);
  done();
});

test("lines that do not meet say so, and a branch or a main that is not one of the lines is refused", async () => {
  const apart = [CROSS[0], { id: 3, name: "track.b", geom: path([[300, 100], [300, 300]]) }];
  const vc = await fakeVectorCraft({ paths: apart, selection: [2, 3] });
  const { panel, done } = await setup(vc);
  const f = panel.snapshot().joint.found;
  assert.equal(f.ok, false);
  assert.match(f.error, /Lines a and b do not meet/);
  done();
  const vc2 = await fakeVectorCraft({ paths: CROSS, selection: [2, 3] });
  const two = await setup(vc2, '{"kind": "join", "main": "a"}');
  await two.panel.propose("they join", "fake");
  assert.match(two.panel.snapshot().message, /does not apply to a join/);
  const vc3 = await fakeVectorCraft({ paths: CROSS, selection: [2, 3] });
  const three = await setup(vc3, '{"kind": "crossing", "main": "z"}');
  await three.panel.propose("z is the main", "fake");
  assert.match(three.panel.snapshot().message, /one of the two lines/);
  assert.deepEqual(steps(vc3), [], "nothing was applied");
});

test("the plug-in's refusal of a joint is shown", async () => {
  const vc = await fakeVectorCraft({ paths: CROSS, selection: [2, 3], rejectJoint: "Lines a and b do not meet." });
  const { panel, done } = await setup(vc, '{"kind": "crossing"}');
  await panel.propose("they cross", "fake");
  await panel.draw();
  assert.match(panel.snapshot().message, /Could not draw: .*do not meet/);
  done();
});

test("a siding off a main: the panel reads it as a branch, and Draw applies a branch with its turnout number", async () => {
  const BRANCH = [
    { id: 2, name: "track.a", geom: path([[0, 0], [1500, 0]]) },
    { id: 3, name: "track.b", geom: path([[700, 0], [1100, 140]]) },
  ];
  const vc = await fakeVectorCraft({ paths: BRANCH, selection: [2, 3] });
  const { panel, done } = await setup(vc, '{"kind": "branch", "frog": 8}');
  const f = panel.snapshot().joint.found;
  assert.deepEqual([f.ok, f.kind], [true, "branch"]);
  assert.match(f.summary, /line b ends on line a, 112 m along it/);
  await panel.propose("b is a siding off a with a number 8", "fake");
  assert.deepEqual(panel.snapshot().pending.changes, ["new: a branch with a No. 8 turnout"]);
  await panel.draw();
  assert.deepEqual(vc.doc.spec.joints.j1, { kind: "branch", frog: 8, lines: ["a", "b"] });
  done();
});

// ---- the new ones: mixing kinds, switching documents, the schema-built cards

test("a road line and a track line together are a level crossing, and Draw stores it", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "road.a" }, { id: 3, name: "track.b" }], selection: [2, 3] });
  const { panel, done } = await setup(vc);
  const s = panel.snapshot();
  assert.deepEqual([s.target.kind, s.target.road, s.target.track], ["level", "a", "b"]);
  assert.equal(s.level.described, false);
  assert.deepEqual(s.level.card.fields.map((f) => f.key), ["treatment"]);
  await panel.editLevel({ treatment: "gated" });
  assert.equal(panel.snapshot().pending.kind, "level");
  await panel.editLevel({ treatment: "drawbridge" });
  assert.match(panel.snapshot().message, /level or gated/);
  await panel.draw();
  assert.deepEqual(steps(vc), ["select.set", SET_LEVEL_CROSSING, "select.all", "plugin.run", "select.set"]);
  assert.deepEqual(vc.doc.spec.level_crossings.lc1, { road: "a", track: "b", treatment: "gated" });
  assert.equal(panel.snapshot().level.described, true);
  done();
});

test("two roads and a track together still get a plain note", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "road.a" }, { id: 4, name: "road.c" }, { id: 3, name: "track.b" }], selection: [2, 3, 4] });
  const { panel, done } = await setup(vc);
  assert.equal(panel.snapshot().target.kind, "other");
  assert.match(panel.snapshot().message, /not supported yet/);
  done();
});

test("switching documents drops a track and a joint proposal", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2] });
  const one = await setup(vc);
  await one.panel.propose("a yard", "fake");
  assert.equal(one.panel.snapshot().pending.kind, "yard");
  vc.doc.title = "Another";
  vc.doc.revision++;
  await one.panel.tick(true);
  assert.equal(one.panel.snapshot().pending, null);
  one.done();
  const vc2 = await fakeVectorCraft({ paths: CROSS, selection: [2, 3] });
  const two = await setup(vc2, '{"kind": "crossing"}');
  await two.panel.propose("they cross", "fake");
  assert.equal(two.panel.snapshot().pending.kind, "joint");
  vc2.doc.title = "Another";
  vc2.doc.revision++;
  await two.panel.tick(true);
  assert.equal(two.panel.snapshot().pending, null);
});

test("a track card edit is checked before it reaches the plug-in", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2] });
  const { panel, done } = await setup(vc);
  await panel.editTrack({ tracks: 99 });
  const s = panel.snapshot();
  assert.match(s.message, /whole number from 1 to 12/);
  assert.equal(s.pending, null);
  assert.deepEqual(steps(vc), [], "no command was sent");
  done();
});

test("the track card is built from the schema, and an edit becomes a proposal that Draw applies", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2] });
  const { panel, done } = await setup(vc);
  const card = panel.snapshot().track.card;
  assert.deepEqual(card.fields.map((f) => f.key), ["tracks", "spacing", "ballast", "ties"]);
  assert.deepEqual([card.fields[0].value, card.fields[2].value], [1, "on"], "the current values fill the card; true shows as on");
  await panel.editTrack({ tracks: 3, ballast: "off" });
  const p = panel.snapshot().pending;
  assert.deepEqual([p.kind, p.spec.tracks, p.spec.ballast, p.source], ["tracks", 3, false, "edit"]);
  assert.deepEqual(p.changes, ["tracks: 1 → 3", "roadbed: on → off"]);
  assert.deepEqual(panel.snapshot().track.card.fields[0].value, 1, "the stored value stays until Draw");
  await panel.draw();
  assert.deepEqual(vc.doc.spec.tracks.t1.tracks, 3);
  assert.equal(vc.doc.spec.tracks.t1.ballast, false);
  done();
});

test("a yard card edit keeps the yard and leaves an empty length out", async () => {
  const stored = { mains: 2, left: 8, right: 4, ends: "merge", frog: 6, spacing: 3.9624, at: 0, ballast: true, ties: true };
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2], spec: { tracks: {}, yards: { t1: stored } } });
  const { panel, done } = await setup(vc);
  assert.deepEqual(panel.snapshot().track.card.fields.map((f) => f.key), ["mains", "left", "right", "ends", "frog", "spacing", "at", "length", "before_end", "ballast", "ties"]);
  await panel.editTrack({ left: 10, length: null });
  const p = panel.snapshot().pending;
  assert.deepEqual([p.kind, p.spec.left, "length" in p.spec], ["yard", 10, false]);
  assert.deepEqual(p.changes, ["spurs on the left: 8 → 10"]);
  done();
});

test("a joint card edit is checked, and a main that is not one of the lines is refused", async () => {
  const vc = await fakeVectorCraft({ paths: CROSS, selection: [2, 3] });
  const { panel, done } = await setup(vc);
  assert.deepEqual(panel.snapshot().joint.card.fields.map((f) => f.key), ["kind", "main", "frog"]);
  await panel.editJoint({ kind: "crossing", main: "z" });
  assert.match(panel.snapshot().message, /one of the two lines/);
  assert.equal(panel.snapshot().pending, null);
  await panel.editJoint({ kind: "crossing", main: "b" });
  const p = panel.snapshot().pending;
  assert.deepEqual([p.kind, p.spec.main, p.source], ["joint", "b", "edit"]);
  assert.deepEqual(steps(vc), []);
  done();
});

test("a plain line in track mode is described as a track, not a road", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "" }], selection: [5] });
  const { panel, done } = await setup(vc, '{"tracks": 2}');
  assert.equal(panel.setMode("track"), true);
  await panel.tick(true);
  assert.equal(panel.snapshot().track.described, false);
  await panel.propose("double track", "fake");
  assert.equal(panel.snapshot().pending.kind, "tracks");
  await panel.draw();
  assert.deepEqual(steps(vc), ["select.set", SET_TRACKS, "select.all", "plugin.run", "select.set"]);
  assert.equal(vc.doc.spec.tracks.t1.tracks, 2);
  done();
});

test("Draw on two lines with no stored joint stores what the card shows, a described joint is only rebuilt", async () => {
  const vc = await fakeVectorCraft({ paths: CROSS, selection: [2, 3] });
  const { panel, done } = await setup(vc);
  assert.equal(panel.snapshot().joint.described, false);
  await panel.draw();
  assert.deepEqual(steps(vc), ["select.set", SET_JOINT, "select.all", "plugin.run", "select.set"]);
  assert.deepEqual(vc.doc.spec.joints.j1, { kind: "crossing", lines: ["a", "b"] });
  assert.equal(panel.snapshot().joint.described, true);
  const before = vc.doc.calls.length;
  await panel.draw();
  assert.ok(!vc.doc.calls.slice(before).some((c) => c.command === SET_JOINT), "a stored joint is not sent again");
  done();
});

test("a second copy of the same document is a different document: proposals do not follow", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2] });
  vc.doc.fileName = "plan.vectorcraft";
  const { panel, done } = await setup(vc);
  await panel.editTrack({ tracks: 2 });
  assert.equal(panel.snapshot().pending.kind, "tracks");
  vc.doc.fileName = "plan copy.vectorcraft"; // the same inner title, another file
  vc.doc.revision++;
  await panel.tick(true);
  assert.equal(panel.snapshot().pending, null);
  done();
});

test("track layers are kept above road layers, so rails run over a road at a level crossing", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "road.a" }, { id: 3, name: "track.b" }], selection: [3] });
  vc.doc.layers.unshift({ id: 3001, name: "track b", visible: true, locked: false, parent: null });
  vc.doc.layers.unshift({ id: 3000, name: "road a", visible: true, locked: false, parent: null });
  const { panel, done } = await setup(vc);
  await panel.draw();
  const tops = vc.doc.layers.filter((l) => l.parent === null).map((l) => l.name);
  assert.ok(tops.indexOf("track b") < tops.indexOf("road a"), `track above road: ${tops}`);
  done();
});

test("lines drawn with the shape tools count as lines: a Line becomes a track, an Ellipse is offered as a plain line", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "", kind: "Line" }, { id: 6, name: "", kind: "Ellipse" }], selection: [5] });
  const { panel, done } = await setup(vc, '{"tracks": 1}');
  assert.equal(panel.snapshot().target.kind, "line", "a shape-tool line is a plain line");
  await trackMode(panel);
  await panel.draw();
  assert.equal(vc.doc.paths.find((p) => p.id === 5).name, "track.t1");
  const layers = vc.doc.layers.map((l) => l.name);
  assert.ok(layers.includes("track t1 spine"), `the shape-tool line is sorted into its layer: ${layers}`);
  await vc.doc.selection.splice(0, 9, 6);
  vc.doc.revision++;
  await panel.tick(true);
  assert.equal(panel.snapshot().target.kind, "line", "an ellipse is a plain line too");
  done();
});


test("a yard saved by 1.5 (millimetres, schema 1) is shown and changed in metres, never as metres of millimetres", async () => {
  const spec = { schema: 1, tracks: {}, yards: { t1: { mains: 1, left: 2, ends: "stub", frog: 6, spacing: 24.765, at: 100, length: 2000, crossovers: [{ from: 1, to: 1, at: 500 }], ballast: true, ties: true } } };
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2], spec });
  const { panel, done } = await setup(vc);
  const s = panel.snapshot().track.spec;
  assert.ok(Math.abs(s.spacing - 3.9624) < 1e-6 && s.at === 16 && s.length === 320, JSON.stringify(s));
  assert.equal(s.crossovers[0].at, 80);
  done();
});

test("any drawing tool's line counts: a rounded rectangle is a line, a group or text is not", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 5, name: "", kind: "Rounded Rectangle" }, { id: 6, name: "", kind: "Type" }], selection: [5] });
  const { panel, done } = await setup(vc);
  assert.equal(panel.snapshot().target.kind, "line", "a rounded rectangle is a line");
  vc.doc.selection.splice(0, 9, 6);
  vc.doc.revision++;
  await panel.tick(true);
  assert.notEqual(panel.snapshot().target?.kind, "line", "text is not a line");
  done();
});

test("after a build the panel dashes every tie line into ties", async () => {
  const spec = { tracks: { t1: { tracks: 1 } }, yards: {} };
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }, { id: 7, name: "gen.track.t1.t1.ties" }, { id: 8, name: "gen.track.t1.t1.rails" }], selection: [2], spec });
  const { panel, done } = await setup(vc, '{"tracks": 1}');
  await panel.draw();
  const call = vc.doc.calls.find((c) => c.command === "stroke.set");
  assert.ok(call, `stroke.set was sent: ${vc.doc.calls.map((c) => c.command)}`);
  assert.deepEqual(call.params.ids, [7], "only the tie lines");
  // The dash comes from the plug-in's tie sizes (core/src/rail_standards.rs), in document points.
  const rs = (await import("node:fs")).readFileSync(new URL("../../core/src/rail_standards.rs", import.meta.url), "utf8");
  const k = (n) => Number(rs.match(new RegExp(`pub const ${n}: f64 = ([0-9.]+);`))[1]);
  const pt = k("PT_PER_MM");
  assert.deepEqual(call.params.dash.map((x) => +x.toFixed(6)), [k("TIE_WIDTH") * pt, (k("TIE_PITCH") - k("TIE_WIDTH")) * pt].map((x) => +x.toFixed(6)));
  assert.equal(call.params.cap, "butt");
  done();
});

test("tie lines built without the panel (the plug-in menu, an agent) are dashed when the panel sees them, once", async () => {
  const vc = await fakeVectorCraft({ paths: [{ id: 2, name: "track.t1" }], selection: [2], spec: { tracks: { t1: { tracks: 1 } }, yards: {} } });
  const { panel, done } = await setup(vc);
  vc.doc.paths.push({ id: 9, name: "gen.track.t1.t1.ties", kind: "Path" }); // a build the panel did not run
  vc.doc.revision++;
  await panel.tick(true);
  const sets = () => vc.doc.calls.filter((c) => c.command === "stroke.set");
  assert.deepEqual(sets().map((c) => c.params.ids), [[9]]);
  vc.doc.revision++; // an undo of the dash, say: the same line again is left as the user made it
  await panel.tick(true);
  assert.equal(sets().length, 1, "a line already dashed once is not dashed again");
  done();
});
