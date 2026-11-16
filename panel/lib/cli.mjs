// Runs the native cartographer-cli (parse, edit, library, schema) and returns its one line of JSON. The CLI exits 1 when ok is false; its reply is still returned.
import { execFile } from "node:child_process";
import { existsSync } from "node:fs";
import { resolve } from "node:path";

const here = import.meta.dirname;

/** Where the CLI binary is: CARTOGRAPHER_CLI, else the release build next to the panel. */
export function cliPath() {
  const p = process.env.CARTOGRAPHER_CLI ?? resolve(here, "../../target/release/cartographer-cli");
  if (!existsSync(p)) throw new Error(`cartographer-cli not found at ${p}. Build it: node scripts/build.mjs (or set CARTOGRAPHER_CLI)`);
  return p;
}

export function runCli(args) {
  return new Promise((done, fail) => {
    execFile(cliPath(), args, { timeout: 10_000 }, (err, stdout) => {
      if (err && !stdout) {
        // The helper did not answer at all (crashed, killed, or could not start): say so in plain words.
        const why = err.killed ? "it took longer than 10 seconds" : typeof err.code === "number" ? `exit code ${err.code}` : (err.code ?? err.message);
        return fail(new Error(`cartographer-cli could not run (${why}); run node scripts/check.mjs`));
      }
      try {
        done(JSON.parse(stdout));
      } catch {
        fail(new Error(`cartographer-cli printed something unexpected: ${stdout.slice(0, 200)}`));
      }
    });
  });
}

/** Checks document settings (JSON text). {ok, settings, summary} or {ok: false, error}. */
export const parseSettings = (jsonText) => runCli(["parse-settings", jsonText]);
/** The fields of a kind of description (for now "settings"): {ok, kind, fields}. */
export const schema = (kind) => runCli(["schema", kind]);

export const parseCrossSection = (text) => runCli(["parse", text]);
export const library = () => runCli(["library"]);
/** op: ["delete", i] | ["move", from, to] | ["width", i, metres] | ["insert", i, "text"] */
export const editCrossSection = (text, op) => runCli(["edit", text, ...op.map(String)]);
/** Checks a junction description (JSON text). {ok, junction, summary} or {ok: false, error}. */
export const parseJunction = (jsonText) => runCli(["parse-junction", jsonText]);
/** What a proposed cross-section changes compared with the current one, as sentences. */
export const diffCrossSection = (current, proposed) => runCli(["diff", current, proposed]);
/** Checks an intersection description (JSON text) for two crossing roads. {ok, crossing, summary} or {ok: false, error}. */
export const parseCrossing = (jsonText) => runCli(["parse-crossing", jsonText]);
/** Checks a hub description (JSON text) for three or more meeting roads. {ok, hub, summary} or {ok: false, error}. */
export const parseHub = (jsonText) => runCli(["parse-hub", jsonText]);
/** Checks a lane transition (JSON text) and, given the road's current cross-section, says what it becomes. {ok, transition, summary, after?} or {ok: false, error}. */
/** Checks a water description (JSON text), optionally against "open" or "closed". {ok, water, summary} or {ok: false, error}. */
export const parseWater = (jsonText, shape) => runCli(["parse-water", jsonText, ...(shape ? [shape] : [])]);
/** Checks an embankment description (JSON text). {ok, embankment, summary} or {ok: false, error}. */
export const parseEmbankment = (jsonText) => runCli(["parse-embankment", jsonText]);
/** Checks a road-meets-water description (JSON text with road and water ids). {ok, wet, summary} or {ok: false, error}. */
export const parseWet = (jsonText) => runCli(["parse-wet", jsonText]);
export const parseTransition = (jsonText, crossSection) => runCli(["parse-transition", jsonText, ...(crossSection ? [crossSection] : [])]);
