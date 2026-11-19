// Converts a saved VectorCraft document made with Road Builder and/or Track Layer into one Cartographer builds. Run it on a COPY of the file's
// folder; the input is never changed and the output is never written over an existing file (unless --force).
//   node scripts/migrate.mjs <in.vectorcraft> <out.vectorcraft> [--force]
// Then open the output in VectorCraft, install Cartographer, select all and press Build. See docs/MIGRATION.md.
import "./lib/require-node.mjs";
import { existsSync, lstatSync, readFileSync, renameSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { migrateBytes } from "./lib/migrate.mjs";

const args = process.argv.slice(2);
const force = args.includes("--force");
const [input, output] = args.filter((a) => a !== "--force");
const fail = (message, code = 1) => {
  console.error(message);
  process.exit(code);
};
if (!input || !output) fail("usage: node scripts/migrate.mjs <in.vectorcraft> <out.vectorcraft> [--force]", 2);

let bytes;
try {
  bytes = readFileSync(resolve(input));
} catch (e) {
  fail(`cannot read ${input}: ${e.code ?? e.message}`);
}
// The same file under another name (a hard link, a symlink, a different capitalisation on a case-insensitive disk) is found by its identity.
const identity = (p) => {
  const s = statSync(p);
  return `${s.dev}:${s.ino}`;
};
if (resolve(input) === resolve(output) || (existsSync(resolve(output)) && identity(resolve(input)) === identity(resolve(output)))) fail("The output must be a different file from the input: the input is never changed.");
// lstat sees a symlink even when it points nowhere.
const present = (p) => {
  try {
    lstatSync(p);
    return true;
  } catch {
    return false;
  }
};
if (present(resolve(output)) && !force) fail(`${output} already exists. Choose another name, or add --force to replace it.`);
if (!existsSync(dirname(resolve(output)))) fail(`The folder ${dirname(resolve(output))} does not exist.`);

let result;
try {
  result = migrateBytes(bytes);
} catch (e) {
  fail(e.message);
}
for (const line of result.report) console.log(line);
if (/^Nothing to migrate/.test(result.report[0])) process.exit(0);
// Written to a new temporary file next to the output, then renamed into place: a failed write never leaves half a file.
const temp = `${resolve(output)}.${process.pid}.tmp`;
try {
  writeFileSync(temp, result.bytes, { flag: "wx" });
  renameSync(temp, resolve(output));
} catch (e) {
  rmSync(temp, { force: true });
  fail(`could not write ${output}: ${e.code ?? e.message}`);
}
console.log(`wrote ${output}`);
