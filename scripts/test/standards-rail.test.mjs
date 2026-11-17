import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { markdown, problems, rust } from "../lib/standards-rail.mjs";

const source = { title: "T", publisher: "P", url: "u", retrieved: "r" };
const db = (entries = [], tables = {}) => ({ title: "Rail", generated: "2026-11-17", note: "n", sources: { s: source }, entries, tables });
const verified = { id: "a", topic: "x", status: "verified", source: "s", section: "1", quote: "q", values: [{ name: "v", value: 1 }] };
const table = (frogs) => ({ source: "s", unit: "in", title: "t", fields: { lead: "l" }, rows: frogs.map((f) => ({ frog: f, lead: 1 })) });

test("a duplicate id is reported", () => {
  assert.ok(problems(db([verified, { ...verified }])).some((p) => /duplicate id/.test(p)));
});

test("a status other than verified or unverified is reported", () => {
  assert.ok(problems(db([{ ...verified, status: "verified-secondary" }])).some((p) => /status must be verified or unverified/.test(p)));
});

test("an unverified entry without a note is reported, and a verified one without a quote", () => {
  assert.ok(problems(db([{ id: "b", topic: "x", status: "unverified" }])).some((p) => /says where its value came from/.test(p)));
  assert.ok(problems(db([{ ...verified, quote: "" }])).some((p) => /quotes its source/.test(p)));
});

test("a table with frogs out of order is reported", () => {
  assert.ok(problems(db([], { t: table([4, 6, 5]) })).some((p) => /ascending and unique/.test(p)));
  assert.deepEqual(problems(db([verified], { t: table([4, 5, 6]) })), []);
});

test("the shipped rail database is valid and produces a document and a Rust table", () => {
  const shipped = JSON.parse(readFileSync(join(import.meta.dirname, "../../library/standards-rail.json"), "utf8"));
  assert.deepEqual(problems(shipped), []);
  assert.match(markdown(shipped), /^# /);
  const rows = shipped.tables.rp12_35.rows.length;
  assert.match(rust(shipped), new RegExp(`pub const ROWS: \\[Row; ${rows}\\]`));
});
