// The rail (NMRA) standards database: its checker, its document and the Rust table it generates. Ported from Track Layer; used by scripts/standards.mjs.
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "../..");
const db = JSON.parse(readFileSync(resolve(root, "library/standards-rail.json"), "utf8"));

export function problems(d = db) {
  const bad = [];
  const ids = new Set();
  for (const e of d.entries) {
    const at = `entry ${e.id}`;
    if (ids.has(e.id)) bad.push(`${at}: duplicate id`);
    ids.add(e.id);
    if (!["verified", "unverified"].includes(e.status)) bad.push(`${at}: status must be verified or unverified`);
    if (e.status === "verified") {
      if (!d.sources[e.source]) bad.push(`${at}: source "${e.source}" is not in sources`);
      if (!e.section) bad.push(`${at}: a verified entry names its section`);
      if (!e.quote) bad.push(`${at}: a verified entry quotes its source`);
    } else if (!e.note) bad.push(`${at}: an unverified entry says where its value came from`);
  }
  for (const [name, t] of Object.entries(d.tables ?? {})) {
    if (!d.sources[t.source]) bad.push(`table ${name}: source "${t.source}" is not in sources`);
    const frogs = t.rows.map((r) => r.frog);
    if (frogs.some((f, i) => i > 0 && f <= frogs[i - 1])) bad.push(`table ${name}: frog numbers must be ascending and unique`);
    for (const r of t.rows) for (const f of ["frog", ...Object.keys(t.fields)]) if (typeof r[f] !== "number") bad.push(`table ${name}, frog ${r.frog}: missing or non-numeric ${f}`);
  }
  for (const [k, s] of Object.entries(d.sources)) for (const f of ["title", "publisher", "url", "retrieved"]) if (!s[f]) bad.push(`source ${k}: missing ${f}`);
  return bad;
}

export function markdown(d = db) {
  const out = [`# ${d.title}`, "", `Generated from library/standards-rail.json (${d.generated}). Do not edit by hand: \`node scripts/standards.mjs md\`.`, "", d.note, "", "## Sources", ""];
  for (const [name, t] of Object.entries(d.tables ?? {})) {
    if (!d.sources[t.source]) bad.push(`table ${name}: source "${t.source}" is not in sources`);
    const frogs = t.rows.map((r) => r.frog);
    if (frogs.some((f, i) => i > 0 && f <= frogs[i - 1])) bad.push(`table ${name}: frog numbers must be ascending and unique`);
    for (const r of t.rows) for (const f of ["frog", ...Object.keys(t.fields)]) if (typeof r[f] !== "number") bad.push(`table ${name}, frog ${r.frog}: missing or non-numeric ${f}`);
  }
  for (const [k, s] of Object.entries(d.sources)) out.push(`- **${k}**: [${s.title}](${s.url}), ${s.publisher}, retrieved ${s.retrieved}`);
  out.push("", "## Entries", "");
  for (const e of d.entries) {
    out.push(`### ${e.topic} (\`${e.id}\`, ${e.status})`, "");
    if (e.status === "verified") out.push(`Source: ${e.source}, ${e.section}.`, "", `> ${e.quote}`, "");
    if (e.note) out.push(e.note, "");
  }
  for (const [name, t] of Object.entries(d.tables ?? {})) {
    out.push(`## Table ${name}: ${t.title}`, "", `Source: ${t.source}. Units: ${t.unit}. Fields:`, "");
    for (const [f, text] of Object.entries(t.fields)) out.push(`- \`${f}\`: ${text}`);
    const cols = ["frog", "lead", "radius", "switch_rail", "heel_angle_deg", "frog_angle_deg"];
    out.push("", cols.join(" | "), cols.map(() => "---").join(" | "));
    for (const r of t.rows) out.push(cols.map((c) => r[c]).join(" | "));
    out.push("");
  }
  return out.join("\n");
}

/** The N turnout table as a Rust file (inches and degrees, as the NMRA prints them). */
export function rust(d = db) {
  const t = d.tables.rp12_35;
  const f = (x) => (Number.isInteger(x) ? `${x}.0` : String(x));
  const rows = t.rows.map((r) => `    Row { frog: ${r.frog}, switch_rail: ${f(r.switch_rail)}, heel_angle_deg: ${f(r.heel_angle_deg)}, lead: ${f(r.lead)}, radius: ${f(r.radius)}, frog_angle_deg: ${f(r.frog_angle_deg)}, gage_points: [[${f(r.x1)}, ${f(r.y1)}], [${f(r.x2)}, ${f(r.y2)}], [${f(r.x3)}, ${f(r.y3)}]], pf_to_icl: ${f(r.pf_to_icl)}, crossover_crossover: ${f(r.crossover_crossover)} },`);
  return `//! Generated from library/standards-rail.json (table rp12_35) by \`node scripts/standards.mjs rs\`. Do not edit by hand.
//! NMRA RP-12.35, N Scale Straight Switch Turnout (revised February 2015). Inches and degrees, as printed.

#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub frog: u32,
    /// (2) switch rail length.
    pub switch_rail: f64,
    /// (5) switch heel angle.
    pub heel_angle_deg: f64,
    /// (8) lead from the point of switch to the theoretical point of frog.
    pub lead: f64,
    /// (11) closure curved rail radius.
    pub radius: f64,
    /// (18) frog angle.
    pub frog_angle_deg: f64,
    /// (13, 12), (15, 14), (17, 16): distance along and offset of three points on the curved closure rail's gage line.
    pub gage_points: [[f64; 2]; 3],
    /// (24) point of frog to intersection of centre lines.
    pub pf_to_icl: f64,
    /// (26) crossover track distance, point of frog to point of frog, for tracks 0.975 in (13 prototype feet) apart.
    pub crossover_crossover: f64,
}

pub const ROWS: [Row; ${t.rows.length}] = [
${rows.join("\n")}
];
`;
}

/** How many entries the shipped rail database has. */
export const entryCount = () => db.entries.length;
