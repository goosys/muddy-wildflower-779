# Haikunator Generator

Generates Heroku-like memorable random strings. Available through the website, REST API, or MCP.

Website: https://haikunator-generator.goosysapp.net/

## Web generator

Select **Generate string** to create a string, then **Copy** to copy it. The page shows generation progress, copy confirmation, and errors. Generated strings are not guaranteed to be unique.

## REST API

No authentication is required.

### JSON

```sh
curl https://haikunator-generator.goosysapp.net/api/gen
```

Example response:

```json
{"name":"cool-rice-4810"}
```

### Plain text

```sh
curl https://haikunator-generator.goosysapp.net/api/gen.txt
```

Example response: `divine-limit-4413`.

## MCP

Connect a Streamable HTTP MCP client to:

```text
https://haikunator-generator.goosysapp.net/mcp
```

No authentication is required. The `gen` tool accepts no arguments and returns one string as structured content, with equivalent JSON text content:

```json
{"name":"cool-rice-4810"}
```

### Example: continuous generation

The website includes a live example below the MCP instructions. Select a count and **Start** to see strings arrive every 500 ms. **Stop** aborts the streaming request.

Use `gen_continuous` with these arguments:

```json
{"count":5,"interval_ms":500}
```

- `count`: 1–20, default 5.
- `interval_ms`: 100–2000 milliseconds, default 500.

The tool waits one interval before each string and returns a finite sequence. Supply `_meta.progressToken` in the MCP call for incremental `notifications/progress` over the POST SSE response. Each generated string is JSON in the progress message. The final structured result has a `results` array. Without a progress token, only the final JSON response is returned.

Example final result for a count of 2:

```json
{"results":[{"name":"divine-limit-4413"},{"name":"wispy-resonance-4355"}]}
```

Closing the streaming request stops generation. Aborting a JSON-only call is not guaranteed to stop server-side generation. Calls last at most 40 seconds. Generated strings are not guaranteed to be unique.

## Terms of Use

This service, including its web interface, API, and MCP endpoint, is free to use.

The service is provided “as is,” without warranties of availability, uptime, reliability, or suitability for any purpose. It may be changed, suspended, or discontinued without prior notice. Generated strings are not guaranteed to be unique.

You use this service at your own risk. To the extent permitted by applicable law, the operator is not liable for any loss or damage arising from the use of, or inability to use, this service. This does not exclude liability that cannot legally be excluded, including liability for intentional misconduct or gross negligence.

## Links

- [Website](https://haikunator-generator.goosysapp.net/)
- [API instructions](https://haikunator-generator.goosysapp.net/#api)
- [MCP instructions](https://haikunator-generator.goosysapp.net/#mcp)
- [Terms of Use](https://haikunator-generator.goosysapp.net/#terms)
- [Powered by Haikunator](https://docs.rs/haikunator/latest/haikunator/struct.Haikunator.html)
- [Source on GitHub](https://github.com/goosys/muddy-wildflower-779)
- [Sitemap](https://haikunator-generator.goosysapp.net/sitemap.xml)
