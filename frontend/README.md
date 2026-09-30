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

For frontend hot reload, start Rsbuild in another terminal:

```sh
pnpm --dir frontend dev
```

Open [http://localhost:5153/](http://localhost:5153/). Requests to `/api` are
proxied to the Worker on port 8787. The Worker also serves the built site at
[http://localhost:8787/](http://localhost:8787/).

## Build and lint

`pnpm build` from the repository root builds the frontend into `.assets` and
compiles the Emscripten Worker. A frontend-only build uses `frontend/dist`:

```sh
pnpm --dir frontend build
pnpm --dir frontend lint
```
