// Builds and packs a release: dist/cartographer-<version>-<system>-<chip>.tar.gz and dist/SHA256SUMS.   node scripts/package.mjs <version>
import "./lib/require-node.mjs";
import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { archiveName, sumsLine } from "./lib/package.mjs";

const version = process.argv[2];
if (!version) {
  console.error("usage: package.mjs <version>");
  process.exit(2);
}
const root = resolve(import.meta.dirname, "..");
const run = (cmd, args) => {
  const r = spawnSync(cmd, args, { cwd: root, stdio: "inherit" });
  if (r.status !== 0) process.exit(r.status ?? 1);
};
run(process.execPath, [join(root, "scripts/build.mjs")]);

const archive = archiveName(version, process.platform, process.arch);
const dist = join(root, "dist");
const stage = join(dist, basename(archive, ".tar.gz"));
rmSync(stage, { recursive: true, force: true });
mkdirSync(join(stage, "bin"), { recursive: true });
cpSync(join(root, "target/wasm32-unknown-unknown/release/cartographer_filter.wasm"), join(stage, "cartographer_filter.wasm"));
cpSync(join(root, "target/release/cartographer-cli"), join(stage, "bin/cartographer-cli"));
for (const d of ["panel", "library", "docs", "scripts"]) if (existsSync(join(root, d))) cpSync(join(root, d), join(stage, d), { recursive: true, filter: (p) => !p.includes("/target/") && !p.endsWith(".DS_Store") });
for (const f of ["README.md", "CHANGELOG.md", "LICENSE"]) if (existsSync(join(root, f))) cpSync(join(root, f), join(stage, f));
run("tar", ["-czf", join(dist, archive), "-C", dist, basename(stage)]);
writeFileSync(join(dist, "SHA256SUMS"), `${sumsLine(archive, readFileSync(join(dist, archive)))}\n`);
console.log(`wrote dist/${archive}`);
