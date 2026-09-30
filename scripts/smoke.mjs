import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const base = new URL(
  process.argv[2] ?? process.env.WORKERS_TEST_URL ?? "http://127.0.0.1:8787",
);
const securityHeaders = JSON.parse(
  await readFile(new URL("../security-headers.json", import.meta.url), "utf8"),
);
const origin = "https://smoke.example";

async function request(path, { status = 200, ...options } = {}) {
  const response = await fetch(new URL(path, base), {
    signal: AbortSignal.timeout(30_000),
    redirect: "manual",
    ...options,
  });
  assert.equal(response.status, status, `${options.method ?? "GET"} ${path}`);
  for (const [name, value] of Object.entries(securityHeaders)) {
    assert.equal(response.headers.get(name), value, `${path}: ${name}`);
  }
  return response;
}

function assertName(name) {
  assert.equal(typeof name, "string");
  const parts = name.split("-");
  assert.equal(parts.length, 3, name);
  assert.ok(parts[0].length > 0 && parts[1].length > 0, name);
  assert.match(parts[2], /^\d{4}$/, name);
}

async function assertApi(path) {
  const response = await request(path, { headers: { Origin: origin } });
  assert.equal(response.headers.get("access-control-allow-origin"), "*");
  assert.equal(response.headers.get("access-control-expose-headers"), null);
  const type = path.endsWith(".txt") ? "text/plain" : "application/json";
  assert.ok(response.headers.get("content-type")?.startsWith(type), path);
  const body = await response.text();
  if (path.endsWith(".txt")) {
    assertName(body);
  } else {
    const json = JSON.parse(body);
    assert.deepEqual(Object.keys(json), ["name"]);
    assertName(json.name);
  }
}

// Exercise the SDK runtime with concurrent requests, including first API access.
await Promise.all(
  Array.from({ length: 20 }, (_, index) =>
    assertApi(index % 2 === 0 ? "/api/gen" : "/api/gen.txt"),
  ),
);
for (let index = 0; index < 3; index += 1) {
  await assertApi("/api/gen");
  await assertApi("/api/gen.txt");
}

for (const path of ["/api/gen", "/api/gen.txt"]) {
  const head = await request(path, { method: "HEAD" });
  assert.equal(await head.text(), "");
  const post = await request(path, { method: "POST", status: 405 });
  const allowed = post.headers.get("allow")?.split(",").map((v) => v.trim());
  assert.ok(allowed?.includes("GET") && allowed.includes("HEAD"), path);

  const preflight = await request(path, {
    method: "OPTIONS",
    headers: {
      Origin: origin,
      "Access-Control-Request-Method": "GET",
      "Access-Control-Request-Headers": "content-type,x-smoke-test",
    },
  });
  assert.equal(preflight.headers.get("access-control-allow-origin"), "*");
  assert.equal(preflight.headers.get("access-control-allow-methods"), "*");
  assert.equal(preflight.headers.get("access-control-allow-headers"), "*");
  assert.equal(preflight.headers.get("access-control-allow-credentials"), null);
}
await request("/api/does-not-exist", { status: 404 });

const index = await request("/");
assert.ok(index.headers.get("content-type")?.startsWith("text/html"));
const html = await index.text();
assert.match(html, /Haikunator Generator/);
const references = [...html.matchAll(/(?:src|href)=["']([^"']+)["']/g)]
  .map((match) => new URL(match[1], base))
  .filter((url) => url.origin === base.origin);
const css = references.filter((url) => url.pathname.endsWith(".css"));
const js = references.filter((url) => url.pathname.endsWith(".js"));
assert.ok(css.length > 0, "index must load CSS");
assert.ok(js.length > 0, "index must load JavaScript");
for (const url of [...css, ...js]) {
  const response = await request(`${url.pathname}${url.search}`);
  const type = response.headers.get("content-type");
  assert.match(type ?? "", url.pathname.endsWith(".css") ? /^text\/css/ : /javascript/);
  assert.ok((await response.text()).length > 0, url.pathname);
}

const favicon = await request("/favicon.ico");
assert.deepEqual(
  [...new Uint8Array(await favicon.arrayBuffer()).slice(0, 4)],
  [0, 0, 1, 0],
);
const ogp = await request("/ogp.png");
assert.ok(ogp.headers.get("content-type")?.startsWith("image/png"));
assert.deepEqual(
  [...new Uint8Array(await ogp.arrayBuffer()).slice(0, 8)],
  [137, 80, 78, 71, 13, 10, 26, 10],
);
const missing = await request("/does-not-exist", { status: 404 });
assert.ok(missing.headers.get("content-type")?.startsWith("text/html"));
assert.match(await missing.text(), /404 Not Found/);

console.log(`Workers smoke tests passed at ${base.origin}`);
