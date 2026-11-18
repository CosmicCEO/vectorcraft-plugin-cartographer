// Converts a saved VectorCraft document made with Road Builder and/or Track Layer into one Cartographer builds. Run it on a COPY of the file's
// folder; the input is never changed and the output is never written over an existing file (unless --force).
//   node scripts/migrate.mjs <in.vectorcraft> <out.vectorcraft> [--force]
// Then open the output in VectorCraft, install Cartographer, select all and press Build. See docs/MIGRATION.md.
import "./lib/require-node.mjs";
import { existsSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
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
const same = existsSync(resolve(output)) && realpathSync(resolve(input)) === realpathSync(resolve(output));
if (same || resolve(input) === resolve(output)) fail("The output must be a different file from the input: the input is never changed.");
if (existsSync(resolve(output)) && !force) fail(`${output} already exists. Choose another name, or add --force to replace it.`);

let result;
try {
  result = migrateBytes(bytes);
} catch (e) {
  fail(e.message);
}
for (const line of result.report) console.log(line);
if (/^Nothing to migrate/.test(result.report[0])) process.exit(0);
writeFileSync(resolve(output), result.bytes);
console.log(`wrote ${output}`);
