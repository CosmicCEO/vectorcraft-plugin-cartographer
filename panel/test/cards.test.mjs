import assert from "node:assert/strict";
import { test } from "node:test";
import { cardModel } from "../lib/cards.mjs";

const schema = {
  kind: "settings",
  fields: [
    { key: "style", label: "Style", type: "choice", options: ["a", "b"], default: "a", help: "How it is painted." },
    { key: "seed", label: "Seed", type: "integer", min: 0, max: 9, default: 1, help: "A number." },
  ],
};

test("a card has one field per schema field, with the current value filled in", () => {
  const card = cardModel(schema, { style: "b" });
  assert.equal(card.fields.length, 2);
  assert.deepEqual(card.fields[0], { key: "style", label: "Style", type: "choice", value: "b", options: ["a", "b"], help: "How it is painted." });
  assert.equal(card.fields[1].value, 1, "a value that is not set shows the default");
  assert.equal(card.fields[1].min, 0);
  assert.equal(card.fields[1].max, 9);
});

test("a choice keeps its whole option list and nothing else leaks in", () => {
  const f = cardModel(schema, {}).fields[0];
  assert.deepEqual(f.options, ["a", "b"]);
  assert.equal(f.default, undefined);
});
