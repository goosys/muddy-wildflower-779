# Development

Run commands from the repository root unless noted otherwise.

## Setup

Use the DevContainer: it installs the tools below and runs `pnpm install --frozen-lockfile` automatically. The workspace is mounted at `/workspace`.

| Tool | Version |
| --- | --- |
| Node.js | 24.21.0 |
| pnpm | 12.8.1 |
| Rust | `beta-2026-09-29` |
| `worker-build` | Pinned workers-rs revision (see manual setup) |

<details>
<summary>Manual setup without the DevContainer</summary>

Install the Node.js and pnpm versions above, plus Python 3 for Emscripten, then run:

```sh
rustup toolchain install beta-2026-09-29 --profile minimal \
  --component rustfmt --component clippy --target wasm32-unknown-emscripten
cargo install worker-build \
  --git https://github.com/cloudflare/workers-rs \
  --rev b57ba6ef8198c65499c2f92b1845cc2412dd6e8c --locked
pnpm install --frozen-lockfile
```

</details>

## Local development

Start the Worker with `pnpm dev`; Wrangler builds it before startup. The first Rust build downloads the Emscripten SDK.

| Task | Command | Result |
| --- | --- | --- |
| Run Worker and built frontend | `pnpm dev` | [localhost:8787](http://localhost:8787/) |
| Frontend hot reload | `pnpm --dir frontend dev` in another terminal | [localhost:5153](http://localhost:5153/); proxies `/api` and `/mcp` to the Worker |
| Build or rebuild after Rust changes | `pnpm build` | Frontend in `.assets` and Emscripten Worker |

## Checks

| Check | Command | Prerequisite |
| --- | --- | --- |
| Rust tests | `cargo test --locked` | No running server needed |
| Rust formatting | `cargo fmt --all -- --check` | — |
| Frontend lint | `pnpm --dir frontend lint` | — |
| HTTP smoke test | `pnpm test` | Worker running with `pnpm dev` |

Run Clippy with the project's warning settings:

```sh
cargo clippy --locked --all-targets -- \
  -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms
```

The smoke test covers REST API, static assets, security headers, and MCP calls, streaming, cancellation, and validation. Set `WORKERS_TEST_URL` to use a different local URL. CI runs Rust checks, the Emscripten build, and the HTTP smoke test.

## Deployment

Authenticate, then deploy:

```sh
pnpm exec wrangler login --device --browser=false
pnpm exec wrangler deploy
```

Wrangler builds and publishes the Worker and frontend together. The Worker name and Custom Domain are configured in `wrangler.toml`.
