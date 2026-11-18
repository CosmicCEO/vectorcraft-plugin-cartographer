// Plain-words differences between a stored description and a proposed one, for the "What changes" list on each card.
const LABEL = {
  treatment: "type", crosswalks: "crosswalks", radius: "kerb radius", minor: "road that stops", over: "road on top", main: "road that stays", diameter: "width",
  stub: "short stub", stub_length: "stub length", pocket: "turn pockets", pocket_length: "pocket length", pocket_taper: "pocket taper", slip: "slip lanes",
  slip_radius: "slip lane radius", opening: "opening", accel_length: "speed-change lane", accel_taper: "speed-change taper", beacon: "beacons", gates: "gates",
  control: "control", length: "length", side: "side", cross_section: "cross-section",
  kind: "kind", name: "name", width: "width", flow: "flow", speed: "speed", bank_left: "left bank height", bank_right: "right bank height", slope: "side slope (H:V)",
  height: "height", from: "starts at", to: "ends at", sides: "sides", ramp: "ramp length", deck_height: "clearance above water", abutment: "abutment beyond banks",
  shape: "culvert shape", count: "number of culverts", size: "culvert size", fill_height: "fill height",
};
const SKIP = new Set(["roads", "road", "water", "piece"]);

const show = (v) => (v === null || v === undefined ? "automatic" : v === true ? "on" : v === false ? "off" : typeof v === "object" ? JSON.stringify(v) : String(v));

/** Sentences for what changes from `before` to `after` (objects). `before` null means there is nothing stored yet. */
export function describeChanges(before, after, what = "intersection") {
  if (!before) return [`a new ${what}`];
  const out = [];
  for (const key of [...new Set([...Object.keys(before), ...Object.keys(after)])]) {
    if (SKIP.has(key)) continue;
    if (key === "legs") {
      const [a, b] = [before.legs ?? [], after.legs ?? []];
      if (JSON.stringify(a) !== JSON.stringify(b)) out.push(`legs: ${a.length} → ${b.length}${b.length ? ` (${b.map((l) => l.side).join(", ")})` : ""}`);
      continue;
    }
    const [x, y] = [before[key], after[key]];
    if (JSON.stringify(x ?? null) !== JSON.stringify(y ?? null)) out.push(`${LABEL[key] ?? key}: ${show(x)} → ${show(y)}`);
  }
  return out.length ? out : ["no change"];
}

// The same for tracks and yards: a description there is {kind, spec}, and a change of kind is said in words.
const TRACK_LABEL = {
  tracks: "tracks", spacing: "spacing (mm)", ballast: "roadbed", ties: "ties", mains: "main tracks", left: "spurs on the left", right: "spurs on the right",
  ends: "far end", frog: "turnout number", crossovers: "crossovers", at: "starts at (mm)", length: "length (mm)",
};
const showTrack = (v) => (v === null || v === undefined ? "automatic" : v === true ? "on" : v === false ? "off" : Array.isArray(v) ? (v.length ? v.map(crossover).join("; ") : "none") : String(v));
const crossover = (c) => `main ${c.from} to ${c.to}${c.at === undefined ? " in the middle" : ` at ${c.at} mm`}`;
const TRACK_KIND = { tracks: "parallel tracks", yard: "a yard" };

/** Sentences for what changes from `before` ({kind, spec}, or null if nothing is stored) to `after`. */
export function describeTrackChanges(before, after) {
  if (!before) return [`new: ${TRACK_KIND[after.kind]}`];
  if (before.kind !== after.kind) return [`changes from ${TRACK_KIND[before.kind]} to ${TRACK_KIND[after.kind]}`];
  const out = [];
  // In the order of the card, then any other field.
  const order = Object.keys(TRACK_LABEL);
  const keys = [...new Set([...Object.keys(before.spec), ...Object.keys(after.spec)])].sort((a, b) => (order.indexOf(a) + 1 || 99) - (order.indexOf(b) + 1 || 99));
  for (const key of keys) {
    const [x, y] = [before.spec[key], after.spec[key]];
    if (JSON.stringify(x ?? null) !== JSON.stringify(y ?? null)) out.push(`${TRACK_LABEL[key] ?? key}: ${showTrack(x)} → ${showTrack(y)}`);
  }
  return out.length ? out : ["no change"];
}
