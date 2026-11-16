// Comparing two SVG exports for geometry: ids and other per-session attributes are ignored, numbers are rounded to 3 decimals.

const round = (n) => {
  const v = Number(Number(n).toFixed(3));
  return Object.is(v, -0) ? "0" : String(v);
};

/** The SVG as a normalised string: no XML header, comments or document title, no `data-*` attributes and no `id` that is not an object name (`gen.*`, `road.*`, `water.*`, `junction.*`), numbers rounded, whitespace between tags dropped. */
export function normalizeSvg(text) {
  return text
    .replace(/<\?xml[\s\S]*?\?>/g, "")
    .replace(/<!--[\s\S]*?-->/g, "")
    .replace(/<title>[\s\S]*?<\/title>/g, "") // the document's name differs per session
    .replace(/\sdata-[\w-]+="[^"]*"/g, "")
    // An object's name is its id and is compared; other ids are per-session counters.
    .replace(/\sid="([^"]*)"/g, (whole, id) => (/^(gen|road|water|junction)(\.|$)/.test(id) ? whole : ""))
    .replace(/-?\d+\.\d+/g, round)
    .replace(/\s+/g, " ")
    .replace(/>\s+</g, "><")
    .replace(/\s+\/>/g, "/>")
    .replace(/\s+>/g, ">")
    .trim();
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
