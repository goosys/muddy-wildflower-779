# Haikunator Generator Frontend

The frontend uses React, TypeScript, Rsbuild, Tailwind CSS, and Biome. Cloudflare
Workers serves the generated frontend and the Axum API.

## Development

Run these commands from the repository root. The DevContainer installs the pnpm
workspace dependencies automatically; outside it, install them first:

```sh
pnpm install --frozen-lockfile
```

Start the local Worker:

```sh
pnpm dev
```

This selects Wrangler's `development` environment, allowing local MCP access on the development ports.

For frontend hot reload, start Rsbuild in another terminal:

```sh
pnpm --dir frontend dev
```

Open [http://localhost:5153/](http://localhost:5153/). Requests to `/api` and `/mcp` are
proxied to the Worker on port 8787. The Worker also serves the built site at
[http://localhost:8787/](http://localhost:8787/).

## Build and lint

`pnpm build` from the repository root builds the frontend into `.assets` and
compiles the Emscripten Worker. A frontend-only build uses `frontend/dist`:

```sh
pnpm --dir frontend build
pnpm --dir frontend lint
```

The public URLs and shared site content live in `frontend/src/content/site.json`.
Build and development commands regenerate the public documents automatically.
See [service settings](../docs/development.md#service-settings) when changing the
deployment URLs.
