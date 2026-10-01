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

`pnpm dev` selects Wrangler's `development` environment, which has no public routes and allows local MCP Hosts and browser Origins. A direct `wrangler dev` command must also include `--env development`. See [MCP access policy](mcp.md#access-policy) for the allowed local ports.

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

Production uses the public MCP access policy and rejects local Hosts and Origins. Only `MCP_ENVIRONMENT=development` enables local access; an unset or unrecognized value uses the public policy.

### Service settings

For a fork or a different deployment, update `url` (the public HTTPS URL) and
`workersDevUrl` (the Worker's `workers.dev` HTTPS URL) in
`frontend/src/content/site.json`, plus the Worker `name` and Custom Domain route
in `wrangler.toml`. Use canonical HTTPS origins without a trailing slash,
explicit port, or path. The build validates the URLs and checks that these
settings agree.

The frontend metadata, public site documents, MCP access policy, and HTTP smoke
test use this shared configuration. The Rust Worker embeds it at compile time,
so run `pnpm build` after changing it. To regenerate only the public documents,
run `node scripts/generate-site-docs.mjs`.

### Preview URLs

To upload a version with a readable preview alias:

```sh
pnpm exec wrangler versions upload --preview-alias staging
```

When Version URLs are enabled for the Worker, its MCP endpoint is
`https://staging-<worker>.<account>.workers.dev/mcp`, using the Worker and account
from `workersDevUrl`. Uploading a version does not change production traffic.

The [MCP access policy](mcp.md#access-policy) automatically recognizes this Worker's branch, deployment, and version preview URLs. See Cloudflare's [Version URLs](https://developers.cloudflare.com/workers/versions-and-deployments/version-urls/) and [Workers Previews](https://developers.cloudflare.com/workers/previews/) for the two workflows. This repository does not enable additional public preview routes in its configuration.
