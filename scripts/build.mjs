import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { unstable_readConfig } from "wrangler";

const root = fileURLToPath(new URL("../", import.meta.url));
const assets = fileURLToPath(new URL("../.assets/", import.meta.url));
const site = JSON.parse(readFileSync(new URL("../frontend/src/content/site.json", import.meta.url), "utf8"));
const publicUrl = new URL(site.url);
const workersUrl = new URL(site.workersDevUrl);
const wrangler = unstable_readConfig({
  config: fileURLToPath(new URL("../wrangler.toml", import.meta.url)),
}, { hideWarnings: true });
const validLabel = label => label.length <= 63 && /^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$/i.test(label);
for (const [key, url] of [["url", publicUrl], ["workersDevUrl", workersUrl]]) {
  const labels = url.hostname.split(".");
  if (url.protocol !== "https:" || url.port || site[key] !== url.origin ||
      url.hostname.length > 253 || labels.length < 2 || !labels.every(validLabel) ||
      /^\d+\.\d+\.\d+\.\d+$/.test(url.hostname)) {
    throw new Error(`site.json ${key} must be a canonical HTTPS origin without a trailing slash or explicit port`);
  }
}
if (!wrangler.routes?.some(route =>
  typeof route === "object" && route.custom_domain && route.pattern === publicUrl.hostname,
)) {
  throw new Error("site.json url must match a Custom Domain route in wrangler.toml");
}
const [worker, account, namespace, tld, ...extra] = workersUrl.hostname.split(".");
if (worker !== wrangler.name || !account || namespace !== "workers" || tld !== "dev" || extra.length) {
  throw new Error("site.json workersDevUrl must use <wrangler.name>.<account>.workers.dev");
}

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
}).map(([name, value]) => `  ${name}: ${value}\n`).join("") +
  "\n/index.md\n  Content-Type: text/markdown; charset=utf-8\n" +
  "\n/sitemap.xml\n  Content-Type: application/xml; charset=utf-8\n");
run("worker-build", ["--emscripten", "--release", "--locked"], {
  ...process.env,
  // Pinned worker-build collects inline JS from Cargo's previous output layout.
  __CARGO_TEMPORARY_BUILD_DIR_NEW_LAYOUT_OPT_OUT: "1",
});
