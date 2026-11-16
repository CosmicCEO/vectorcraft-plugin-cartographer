import assert from "node:assert/strict";
import { test } from "node:test";
import { compareSvg, normalizeSvg } from "../lib/svgcompare.mjs";

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
