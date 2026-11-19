// Comparing two SVG exports for geometry: ids and other per-session attributes are ignored, numbers are rounded to 3 decimals.

const round = (n) => {
  const v = Number(Number(n).toFixed(3));
  return Object.is(v, -0) ? "0" : String(v);
};

/** The SVG with the per-session parts removed (XML header, comments, document title, `data-*` attributes and every `id` that is not an object name), numbers untouched. */
function stripSvg(text) {
  return text
    .replace(/<\?xml[\s\S]*?\?>/g, "")
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/<title>[\s\S]*?<\/title>/g, "") // the document's name differs per session
    .replace(/\sdata-[\w-]+="[^"]*"/g, "")
    // An object's name is its id and is compared; other ids are per-session counters.
    .replace(/\sid="([^"]*)"/g, (whole, id) => (/^(gen|road|water|junction|track)(\.|$)/.test(id) ? whole : ""))
    .replace(/\s+/g, " ")
    .replace(/>\s+</g, "><")
    .replace(/\s+\/>/g, "/>")
    .replace(/\s+>/g, ">")
    .trim();
}

/** The SVG as a normalised string: stripped (see `stripSvg`) and numbers rounded to 3 decimals. */
export function normalizeSvg(text) {
  return stripSvg(text).replace(/-?\d+\.\d+/g, round);
}

const tokens = (text) => normalizeSvg(text).match(/<[^>]*>|[^<]+/g) ?? [];

/** `{same, summary}`: summary is `identical`, or names the first element that differs (or the element counts). */
export function compareSvg(a, b) {
  const [x, y] = [tokens(a), tokens(b)];
  for (let i = 0; i < Math.min(x.length, y.length); i++) {
    if (x[i] !== y[i]) return { same: false, summary: `element ${i} differs: ${x[i].slice(0, 120)} against ${y[i].slice(0, 120)}` };
  }
  if (x.length !== y.length) return { same: false, summary: `different element count: ${x.length} elements against ${y.length}` };
  return { same: true, summary: "identical" };
}

// Attributes whose numbers are lengths: they scale with the drawing. Everything else (a miter limit, an opacity) must match as it is.
const SCALED = new Set(["d", "points", "x", "y", "x1", "y1", "x2", "y2", "cx", "cy", "r", "rx", "ry", "width", "height", "stroke-width"]);
const NUMBER = /-?\d*\.?\d+(?:e-?\d+)?/gi;

/** A tag as its text with every attribute number replaced by `#`, and the numbers in order, each marked as a length or not. */
function pick(tag) {
  const nums = [];
  const skeleton = tag.replace(/([\w:-]+)="([^"]*)"/g, (_, attr, value) => `${attr}="${value.replace(NUMBER, (n) => (nums.push({ v: Number(n), length: SCALED.has(attr) }), "#"))}"`);
  return { skeleton, nums };
}

/**
 * Compares the same drawing made at two scales: every length in `oldText` is multiplied by `factor` and must match the one in `newText` within
 * `tolerance`; names, structure and numbers that are not lengths must match as they are. The `<svg>` tag (the page size) is not compared.
 * `{same, summary}` as for `compareSvg`.
 */
export function compareSvgScaled(oldText, newText, { factor, tolerance }) {
  const split = (t) => (stripSvg(t).match(/<[^>]*>|[^<]+/g) ?? []).filter((tag) => !/^<svg[\s>]/.test(tag));
  const [x, y] = [split(oldText), split(newText)];
  for (let i = 0; i < Math.min(x.length, y.length); i++) {
    const [p, q] = [pick(x[i]), pick(y[i])];
    const shown = () => `element ${i} differs: ${x[i].slice(0, 120)} against ${y[i].slice(0, 120)}`;
    if (p.skeleton !== q.skeleton || p.nums.length !== q.nums.length) return { same: false, summary: shown() };
    for (let k = 0; k < p.nums.length; k++) {
      const [a, b] = [p.nums[k], q.nums[k]];
      const gap = a.length ? Math.abs(a.v * factor - b.v) : Math.abs(a.v - b.v);
      if (gap > (a.length ? tolerance : 0.0015)) return { same: false, summary: `${shown()} (a length is ${gap.toFixed(4)} off)` };
    }
  }
  if (x.length !== y.length) return { same: false, summary: `different element count: ${x.length} elements against ${y.length}` };
  return { same: true, summary: "identical" };
}

const SUBPATH = /M(-?[\d.]+) (-?[\d.]+)((?:(?!M)[^"Z])*)Z/g;

/**
 * Track Layer 0.9.1 wrote a closed smooth ring whose first point is repeated as `M p L p Z`: a point, not a shape (its curve thinner collapsed the ring).
 * Cartographer drops the repeated point and draws the ring. For the parity gate, every such collapsed ring in `oldText` is removed, together with the
 * ring in `newText` that starts where it was (scaled by `factor`). `{old, new, skipped}`: the texts to compare and how many rings were skipped.
 */
export function skipCollapsedRings(oldText, newText, factor, tolerance = 0.001) {
  const all = (text) => [...text.matchAll(SUBPATH)].map((m) => ({ at: m.index, len: m[0].length, x: Number(m[1]), y: Number(m[2]), rest: m[3].trim(), raw: m }));
  const collapsed = all(oldText).filter((m) => m.rest === `L${m.raw[1]} ${m.raw[2]}`);
  const rings = all(newText);
  const used = new Set();
  const pairs = [];
  // Each collapsed ring is matched with exactly one new ring that starts where it was; an unmatched one stays, and so does any extra new ring.
  for (const c of collapsed) {
    const k = rings.findIndex((r, i) => !used.has(i) && Math.abs(r.x - c.x * factor) <= tolerance && Math.abs(r.y - c.y * factor) <= tolerance);
    if (k < 0) continue;
    used.add(k);
    pairs.push([c, rings[k]]);
  }
  const cut = (text, spans) => spans.sort((p, q) => q.at - p.at).reduce((t, m) => t.slice(0, m.at) + t.slice(m.at + m.len), text);
  return { old: cut(oldText, pairs.map((p) => p[0])), new: cut(newText, pairs.map((p) => p[1])), skipped: pairs.length };
}
