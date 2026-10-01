# Haikunator Generator

A web service that generates Heroku-like memorable random strings.

Built with Rust for `wasm32-unknown-emscripten` and runs on Cloudflare Workers.

![Screenshot](docs/screenshot_top.png)

## Usage

Open [Haikunator Generator](https://haikunator-generator.goosysapp.net/),
select **Generate string**, then **Copy** to copy the generated string.
Generated strings are not guaranteed to be unique.

### REST API

Example strings below were obtained from the deployed MCP server.

```sh
# JSON: {"name":"cool-rice-4810"}
curl https://haikunator-generator.goosysapp.net/api/gen

# Plain text: divine-limit-4413
curl https://haikunator-generator.goosysapp.net/api/gen.txt
```

### MCP

Connect a Streamable HTTP MCP client to
`https://haikunator-generator.goosysapp.net/mcp`. No authentication is required.

- `gen`: Generate one string; no arguments required.
- `gen_continuous`: Generate multiple strings with `count` (1–20, default 5)
  and `interval_ms` (100–2000, default 500).

See [MCP usage and behavior](docs/mcp.md) for examples, streaming, and cancellation.

## Documentation

- [Development, deployment, and checks](docs/development.md)
- [Architecture and dependencies](docs/architecture.md)
