import assert from "node:assert/strict";
import { test } from "node:test";
import { compareSvg, compareSvgScaled, normalizeSvg, skipCollapsedRings } from "../lib/svgcompare.mjs";

test("normalizeSvg ignores ids and rounds numbers", () => {
  const a = normalizeSvg('<?xml version="1.0"?>\n<path id="a1" d="M1.00004 2"/>');
  const b = normalizeSvg('<path   id="b9" data-x="7" d="M1 2.0000"/>');
  assert.equal(a, b);
  assert.equal(normalizeSvg('<path d="M-0.0001 5.12345"/>'), normalizeSvg('<path d="M0 5.123"/>'));
});

test("object names in ids are compared; other ids are not", () => {
  const a = '<svg><path id="gen.road.r1.1.surface.0.lane" d="M1 2"/></svg>';
  const b = '<svg><path id="gen.road.r1.1.surface.1.lane" d="M1 2"/></svg>';
  assert.equal(compareSvg(a, b).same, false, "a renamed object differs");
  assert.match(compareSvg(a, b).summary, /gen\.road\.r1\.1\.surface/);
  assert.equal(compareSvg('<svg><path id="gen.road.a" d="M1 2"/></svg>', '<svg><path id="gen.road.a" d="M1 2"/></svg>').same, true);
  assert.equal(compareSvg('<svg><clipPath id="clip7"/></svg>', '<svg><clipPath id="clip9"/></svg>').same, true, "session counters are ignored");
});

test("compareSvg says same for equal geometry", () => {
  const r = compareSvg('<svg><path d="M1 2"/></svg>', '<svg>\n<path id="q" d="M1.0 2.0"/>\n</svg>');
  assert.equal(r.same, true);
  assert.equal(r.summary, "identical");
});

test("compareSvg names the first differing element", () => {
  const r = compareSvg('<svg><path d="M1 2"/><path d="M3 4"/></svg>', '<svg><path d="M1 2"/><path d="M3 9"/></svg>');
  assert.equal(r.same, false);
  assert.match(r.summary, /element 2/);
  assert.match(r.summary, /M3 4/);
});

test("compareSvg reports a different element count when one is a prefix of the other", () => {
  const r = compareSvg("<g/>", "<g/><g/>");
  assert.equal(r.same, false);
  assert.match(r.summary, /different element count: 1 elements against 2/);
});

test("the document title is a per-session name, not geometry", () => {
  const a = '<svg><title>Untitled-2</title><path d="M1 2"/></svg>';
  const b = '<svg><title>Untitled-3</title><path d="M1 2"/></svg>';
  assert.equal(compareSvg(a, b).same, true);
  assert.equal(compareSvg('<svg><title>x</title><path d="M1 2"/></svg>', '<svg><title>x</title><path d="M1 3"/></svg>').same, false, "a real geometry difference still shows");
});

const scaled = (a, b, tolerance = 0.001, factor = 0.2) => compareSvgScaled(a, b, { factor, tolerance });

test("a drawing at another scale is the same drawing", () => {
  const old = '<svg width="10" height="10"><polygon points="0,0 10,20 5,5"/><path d="M0 0 L10 20" stroke-width="2" stroke-miterlimit="4"/></svg>';
  const now = '<svg width="2" height="2"><polygon points="0,0 2,4 1,1"/><path d="M0 0 L2 4" stroke-width="0.4" stroke-miterlimit="4"/></svg>';
  const r = scaled(old, now);
  assert.equal(r.same, true, r.summary);
});

test("a difference beyond the tolerance is reported and names the element", () => {
  const r = scaled('<svg><polygon points="0,0 20,20"/><polygon points="0,0 10,20"/></svg>', '<svg><polygon points="0,0 4,4"/><polygon points="0,0 2.01,4"/></svg>');
  assert.equal(r.same, false);
  assert.match(r.summary, /element 1/, "the second polygon (the page tag is not counted)");
  assert.equal(scaled('<svg><polygon points="0,0 10,20"/></svg>', '<svg><polygon points="0,0 2.0005,4"/></svg>').same, true, "within the tolerance");
});

test("names and structure still have to match at any scale", () => {
  const a = '<svg><path id="gen.track.a.rails" d="M0 0 L10 10"/></svg>';
  assert.equal(scaled(a, '<svg><path id="gen.track.b.rails" d="M0 0 L2 2"/></svg>').same, false, "a renamed object");
  assert.equal(scaled(a, '<svg><path id="gen.track.a.rails" d="M0 0 L2 2"/><path d="M0 0 L1 1"/></svg>').same, false, "an extra element");
  assert.equal(scaled('<svg><path d="M0 0 L10 10" stroke-miterlimit="4"/></svg>', '<svg><path d="M0 0 L2 2" stroke-miterlimit="5"/></svg>').same, false, "a number that is not a length is not scaled");
  assert.equal(scaled('<svg><path d="M0 0 L10 10"/></svg>', '<svg><path d="M0 0 L2 2 L3 3"/></svg>').same, false, "a different number of points");
});

test("the page size is not compared when scales differ", () => {
  assert.equal(scaled('<svg width="100" height="50" viewBox="0 0 100 50"><path d="M1 1"/></svg>', '<svg width="9" height="9" viewBox="0 0 9 9"><path d="M0.2 0.2"/></svg>').same, true);
});

const ring = (d) => `<svg><path id="gen.track.t1.ballast" d="${d}"/></svg>`;

test("a ring Track Layer collapsed to a point is skipped on both sides, and counted", () => {
  const old = ring("M0 0 L10 0 L10 5 L0 5 Z M50 20 L50 20 Z");
  const now = ring("M0 0 L2 0 L2 1 L0 1 Z M10 4 L12 4 L12 3 L10 3 Z");
  const r = skipCollapsedRings(old, now, 0.2);
  assert.equal(r.skipped, 1);
  assert.equal(compareSvgScaled(r.old, r.new, { factor: 0.2, tolerance: 0.001 }).same, true);
});

test("without a collapsed ring nothing is skipped", () => {
  const a = ring("M0 0 L10 0 L10 5 Z");
  const r = skipCollapsedRings(a, ring("M0 0 L2 0 L2 1 Z"), 0.2);
  assert.equal(r.skipped, 0);
  assert.equal(r.old, a);
});

test("only the new ring that starts where the old one collapsed is skipped", () => {
  const r = skipCollapsedRings(ring("M50 20 L50 20 Z"), ring("M10 4 L12 4 L12 3 Z M30 30 L31 30 L31 29 Z"), 0.2);
  assert.equal(r.skipped, 1);
  assert.match(r.new, /M30 30/);
  assert.doesNotMatch(r.new, /M10 4/);
});

test("a collapsed ring skips exactly one new ring, so an extra one at the same place is still seen", () => {
  const r = skipCollapsedRings(ring("M50 20 L50 20 Z"), ring("M10 4 L12 4 L12 3 Z M10 4 L13 4 L13 2 Z"), 0.2);
  assert.equal(r.skipped, 1);
  assert.match(r.new, /M10 4 L13 4/, "the second ring at the same point stays and will differ");
});

test("a collapsed ring with no new ring to match is not skipped", () => {
  const r = skipCollapsedRings(ring("M50 20 L50 20 Z"), ring("M1 1 L2 1 L2 2 Z"), 0.2);
  assert.equal(r.skipped, 0);
  assert.match(r.old, /M50 20 L50 20 Z/);
});

