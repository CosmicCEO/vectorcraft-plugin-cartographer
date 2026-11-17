import assert from "node:assert/strict";
import { test } from "node:test";
import { cardModel } from "../lib/cards.mjs";
import { schema } from "../lib/cli.mjs";
import { KINDS, kindOfLine } from "../lib/kinds.mjs";

test("kindOfLine says road or track or null", () => {
  assert.equal(kindOfLine("road.a"), "road");
  assert.equal(kindOfLine("road"), "road");
  assert.equal(kindOfLine("track.t1"), "track");
  for (const name of ["", "track", "track.", "gen.track.t1.rails", "gen.road.a.1.paving", "water.c", "junction.x", "something"]) assert.equal(kindOfLine(name), null, name);
});

test("each kind says its line, its generated shapes and its layer", () => {
  assert.equal(KINDS.road.line("road.a"), "a");
  assert.equal(KINDS.track.line("track.t1"), "t1");
  assert.ok(KINDS.track.generated.test("gen.track.t1.rails") && !KINDS.track.generated.test("gen.road.a.1.paving"));
  assert.equal(KINDS.track.layer("t1"), "track t1");
  assert.equal(KINDS.road.layer("a"), "road a");
});

test("a card model from the tracks schema fills defaults, and true or false shows as on or off", async () => {
  const card = cardModel(await schema("tracks"), {});
  assert.deepEqual(card.fields.map((f) => [f.key, f.value]), [["tracks", 1], ["spacing", 24.765], ["ballast", "on"], ["ties", "on"]]);
  const off = cardModel(await schema("tracks"), { ballast: false, ties: true });
  assert.deepEqual(off.fields.filter((f) => f.type === "choice").map((f) => f.value), ["off", "on"]);
});

test("the yard and joint schemas make cards too", async () => {
  const yard = cardModel(await schema("yard"), { mains: 3 });
  assert.equal(yard.fields.find((f) => f.key === "mains").value, 3);
  assert.equal(yard.fields.find((f) => f.key === "length").value, null, "an empty length is null");
  const joint = cardModel(await schema("joint"), {});
  assert.deepEqual(joint.fields.map((f) => f.value), ["crossing", "", null]);
});
