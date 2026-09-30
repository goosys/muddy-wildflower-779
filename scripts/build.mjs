import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const assets = fileURLToPath(new URL("../.assets/", import.meta.url));
function run(command, args, env = process.env) {
  const result = spawnSync(command, args, { cwd: root, env, stdio: "inherit" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

rmSync(assets, { recursive: true, force: true });
run("pnpm", ["--dir", "frontend", "build"], {
  ...process.env,
  WORKERS_FRONTEND_DIST: assets,
});
mkdirSync(assets, { recursive: true });
const security = JSON.parse(readFileSync(new URL("../security-headers.json", import.meta.url), "utf8"));
writeFileSync(`${assets}/_headers`, "/*\n" + Object.entries({
  ...security,
  "Access-Control-Allow-Origin": "*",
}).map(([name, value]) => `  ${name}: ${value}\n`).join(""));
run("worker-build", ["--emscripten", "--release", "--locked"], {
  ...process.env,
  // Pinned worker-build collects inline JS from Cargo's previous output layout.
  __CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT: "1",
});
