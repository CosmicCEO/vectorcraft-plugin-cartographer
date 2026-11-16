// Import this first in a script: on an old Node it prints one plain line and stops, instead of failing later with a raw error.
import { nodeProblem } from "./node-check.mjs";

const problem = nodeProblem(process.version);
if (problem) {
  console.error(`fix: ${problem}`);
  process.exit(1);
}
