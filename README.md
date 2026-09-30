# Haikunator Generator

A web service that generates Heroku-like memorable random strings. The Rust
application uses Axum and Tokio, builds for `wasm32-unknown-emscripten`, and runs
on Cloudflare Workers. Workers Static Assets serves the React frontend.

![Screenshot](docs/screenshot_top.png)

## Usage

Public service: [https://haikunator-generator.goosysapp.net/](https://haikunator-generator.goosysapp.net/)

```sh
# JSON
curl https://haikunator-generator.goosysapp.net/api/gen
# {"name":"falling-disk-1736"}

# Plain text
curl https://haikunator-generator.goosysapp.net/api/gen.txt
# broken-wildflower-1928
```

The local development commands below do not deploy the application or change
the public service.

### MCP

The same Worker exposes a stateless Streamable HTTP MCP server at `/mcp`, using
the official Rust SDK `rmcp` on Axum and Tokio. The React page and existing
`/api/gen` and `/api/gen.txt` endpoints remain available.

Connect an MCP client to `http://localhost:8787/mcp` during local development.
After deploying this change, the endpoint is
`https://haikunator-generator.goosysapp.net/mcp`.

The `gen` tool is titled "Haikunator Generator", matching this project and the
existing `/api/gen` interface. It takes no arguments and returns one Heroku-like
memorable random string as structured content: `{"name":"falling-disk-1736"}`,
with equivalent JSON text content. Generated strings are not guaranteed to be
unique.

The server uses JSON responses without session IDs or a persistent SSE stream.
It supports initialization and tool calls from existing Streamable HTTP clients.
Like the existing API, it generates random strings without authentication.
MCP requests with a browser Origin must come from the configured public URLs or local ports
8787/5153; requests without Origin are accepted for native MCP clients.

To try the tool with the official MCP Inspector while `pnpm dev` is running:

```sh
pnpm dlx @modelcontextprotocol/inspector@2.8.0 --cli \
  http://localhost:8787/mcp --transport http --method tools/call \
  --tool-name gen
```

## Development

Open the repository in its DevContainer. It provides Node.js 24.21.0,
pnpm 12.8.1, Rust `beta-2026-09-29`, and the pinned `worker-build`. The workspace
is mounted at `/workspace`; container setup runs `pnpm install --frozen-lockfile`
and leaves server startup to you.

Run from the repository root:

```sh
pnpm build
pnpm dev
```

Open [http://localhost:8787/](http://localhost:8787/). `pnpm build` builds the
frontend into `.assets` and runs `worker-build --emscripten --release --locked`.
The first Rust build downloads the Emscripten SDK. Wrangler also runs the build
command when `pnpm dev` starts.

For frontend changes with hot reload, keep Wrangler running and start Rsbuild in
another terminal:

```sh
pnpm --dir frontend dev
```

Open [http://localhost:5153/](http://localhost:5153/). Rsbuild proxies `/api` to
Wrangler on port 8787. Both ports are exposed by the DevContainer. Rebuild the
Worker after Rust changes.

### Without the DevContainer

Install Node.js 24.21.0, pnpm 12.8.1, and Python 3 for the Emscripten SDK, then install the pinned Rust tools and
workspace dependencies from the repository root:

```sh
rustup toolchain install beta-2026-09-29 --profile minimal \
  --component rustfmt --component clippy --target wasm32-unknown-emscripten
cargo install worker-build \
  --git https://github.com/cloudflare/workers-rs \
  --rev b57ba6ef8198c65499c2f92b1845cc2412dd6e8c --locked
pnpm install --frozen-lockfile
```

Use the same `pnpm build` and `pnpm dev` commands. The root `rust-toolchain.toml`
selects the fixed beta, and `pnpm-workspace.yaml` includes the frontend under one
root lockfile.

## Deployment

Authenticate with Cloudflare from the DevContainer:

```sh
pnpm exec wrangler login --device --browser=false
```

Then deploy from the repository root:

```sh
pnpm exec wrangler deploy
```

Wrangler runs `pnpm build` and publishes the Worker and frontend together. The
Worker is named `haikunator-generator`. Its Custom Domain is configured in
`wrangler.toml`; Cloudflare manages the DNS record and certificate for
`haikunator-generator.goosysapp.net`. The `workers.dev` URL remains enabled.

## Checks

Run the Rust tests and checks from the repository root:

```sh
cargo test --locked
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- \
  -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms
```

These host tests exercise the Axum Router without starting a server. To check
the running Worker, use another terminal while `pnpm dev` is running:

```sh
pnpm test
```

The HTTP smoke test checks JSON and text API responses, concurrent requests,
HEAD/405/404 responses, CORS, security headers, and static assets. It also checks
the MCP lifecycle, tool discovery and calls, argument validation, concurrent
MCP/API requests, Origin validation, and the MCP request size limit. Set
`WORKERS_TEST_URL` to test a different local URL. CI runs the Rust checks, the
Emscripten build, and the local HTTP smoke test.

## Experimental build details

The SDK and `worker-build` share the fixed revision
`b57ba6ef8198c65499c2f92b1845cc2412dd6e8c`. The Tokio patch is pinned in
`Cargo.toml`. Update these dependencies together with the Rust toolchain.

The build script selects Cargo's previous output layout for the pinned
`worker-build` using the
[temporary Cargo compatibility option](https://github.com/rust-lang/cargo/pull/16807).
The Worker entry point buffers API and MCP responses up to 64 KiB to avoid an issue in
the experimental SDK's streaming response bridge.

`security-headers.json` shares the development CSP and GitHub security-header
preset between API and static responses.

## Dependencies

| Dependency | Project |
| --- | --- |
| Rust | [rust-lang.org](https://www.rust-lang.org/) |
| Axum | [tokio-rs/axum](https://github.com/tokio-rs/axum) |
| Tokio | [tokio.rs](https://tokio.rs/) |
| Cloudflare Workers SDK | [cloudflare/workers-rs](https://github.com/cloudflare/workers-rs) |
| Haikunator | [nishanths/rust-haikunator](https://github.com/nishanths/rust-haikunator) |
| MCP Rust SDK | [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) |
