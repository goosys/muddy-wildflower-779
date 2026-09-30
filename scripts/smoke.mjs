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

const mcpHeaders = {
  "Content-Type": "application/json",
  Accept: "application/json, text/event-stream",
  "MCP-Protocol-Version": "2025-11-25",
  Origin: base.origin,
};
let mcpRequestId = 0;
async function mcp(method, params = {}) {
  const id = ++mcpRequestId;
  const response = await request("/mcp", {
    method: "POST",
    headers: mcpHeaders,
    body: JSON.stringify({ jsonrpc: "2.0", id, method, params }),
  });
  assert.ok(response.headers.get("content-type")?.startsWith("application/json"));
  assert.equal(response.headers.get("mcp-session-id"), null);
  const message = await response.json();
  assert.equal(message.jsonrpc, "2.0");
  assert.equal(message.id, id);
  return message;
}

function assertMcpGeneratedString(message) {
  assert.equal(message.error, undefined);
  const result = message.result;
  assert.equal(result.isError, false);
  assert.deepEqual(Object.keys(result.structuredContent), ["name"]);
  assertName(result.structuredContent.name);
  assert.deepEqual(JSON.parse(result.content[0].text), result.structuredContent);
}

const initialized = await mcp("initialize", {
  protocolVersion: "2025-11-25",
  capabilities: {},
  clientInfo: { name: "haikunator-smoke", version: "1.0.0" },
});
assert.equal(initialized.result.protocolVersion, "2025-11-25");
assert.equal(initialized.result.serverInfo.name, "haikunator-generator");
assert.ok(initialized.result.capabilities.tools);
const notification = await request("/mcp", {
  method: "POST",
  headers: mcpHeaders,
  status: 202,
  body: JSON.stringify({ jsonrpc: "2.0", method: "notifications/initialized" }),
});
assert.equal(await notification.text(), "");
const listed = await mcp("tools/list");
assert.deepEqual(listed.result.tools.map((tool) => tool.name), ["gen"]);
assert.equal(listed.result.tools[0].title, "Haikunator Generator");
assert.equal(listed.result.tools[0].annotations.readOnlyHint, true);
assert.match(listed.result.tools[0].description, /Heroku-like memorable random string/);
assert.equal(listed.result.tools[0].inputSchema.type, "object");
assert.deepEqual(listed.result.tools[0].inputSchema.properties ?? {}, {});
assert.equal(listed.result.tools[0].inputSchema.additionalProperties, false);
assertMcpGeneratedString(await mcp("tools/call", { name: "gen", arguments: {} }));
assertMcpGeneratedString(await mcp("tools/call", { name: "gen" }));
await Promise.all(Array.from({ length: 10 }, async () => {
  assertMcpGeneratedString(await mcp("tools/call", { name: "gen", arguments: {} }));
  await assertApi("/api/gen");
}));
for (const arguments_ of [{ count: 1 }, { delimiter: "-" }, { unknown: true }]) {
  const message = await mcp("tools/call", { name: "gen", arguments: arguments_ });
  assert.ok(message.error?.code === -32602 || message.result?.isError === true);
}
for (const method of ["GET", "DELETE"]) {
  await request("/mcp", { method, headers: mcpHeaders, status: 405 });
}
await request("/mcp", {
  method: "POST",
  headers: { ...mcpHeaders, Origin: origin },
  body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "ping" }),
  status: 403,
});
await request("/mcp", {
  method: "POST",
  headers: mcpHeaders,
  body: JSON.stringify({ jsonrpc: "2.0", id: 1, method: "ping", params: { padding: "x".repeat(17 * 1024) } }),
  status: 413,
});
const mcpPreflight = await request("/mcp", {
  method: "OPTIONS",
  headers: {
    Origin: base.origin,
    "Access-Control-Request-Method": "POST",
    "Access-Control-Request-Headers": "content-type,mcp-protocol-version",
  },
});
assert.equal(mcpPreflight.headers.get("access-control-allow-origin"), "*");

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
const faviconSvg = await request("/favicon.svg");
assert.ok(faviconSvg.headers.get("content-type")?.startsWith("image/svg+xml"));
assert.match(await faviconSvg.text(), /Haikunator dice/);
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
