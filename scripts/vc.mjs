// Calls a running VectorCraft's control port and prints the result.
//   node scripts/vc.mjs <method> [json-params]     e.g.  node scripts/vc.mjs engine.execute '{"command":"select.all"}'
import "./lib/require-node.mjs";
import { call } from "./lib/control.mjs";

const [method, raw] = process.argv.slice(2);
if (!method) {
  console.error("usage: vc.mjs <method> [json-params]");
  process.exit(2);
}
try {
  console.log(JSON.stringify(await call(method, raw ? JSON.parse(raw) : {})));
} catch (e) {
  console.error(e.message);
  process.exit(1);
}
