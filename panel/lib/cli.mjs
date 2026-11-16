// Runs the native road-builder-cli (parse, edit, library) and returns its one line of JSON.
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
      if (err && !stdout) return fail(err);
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
