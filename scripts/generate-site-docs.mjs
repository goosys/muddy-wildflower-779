import { mkdirSync, readFileSync, writeFileSync } from "node:fs";

const site = JSON.parse(readFileSync(new URL("../frontend/src/content/site.json", import.meta.url), "utf8"));
site.url = new URL(site.url).origin;
const destination = new URL("../frontend/public/", import.meta.url);
mkdirSync(destination, { recursive: true });
const markdown = `# Haikunator Generator

${site.tagline} Available through the website, REST API, or MCP.

Website: ${site.url}/

## Web generator

Select **Generate string** to create a string, then **Copy** to copy it. The page shows generation progress, copy confirmation, and errors. Generated strings are not guaranteed to be unique.

## REST API

No authentication is required.

### JSON

\`\`\`sh
curl ${site.url}/api/gen
\`\`\`

Example response:

\`\`\`json
{"name":"cool-rice-4810"}
\`\`\`

### Plain text

\`\`\`sh
curl ${site.url}/api/gen.txt
\`\`\`

Example response: \`divine-limit-4413\`.

## MCP

Connect a Streamable HTTP MCP client to:

\`\`\`text
${site.url}/mcp
\`\`\`

No authentication is required. The \`gen\` tool accepts no arguments and returns one string as structured content, with equivalent JSON text content:

\`\`\`json
{"name":"cool-rice-4810"}
\`\`\`

Production accepts browser MCP requests from the configured HTTPS Origins. Workers previews also accept their own HTTPS Origin. Localhost is allowed only in the development configuration; native MCP clients may omit Origin.

### Example: continuous generation

The website includes a live example below the MCP instructions. Select a count and **Start** to see strings arrive every 500 ms. **Stop** aborts the streaming request.

Use \`gen_continuous\` with these arguments:

\`\`\`json
{"count":5,"interval_ms":500}
\`\`\`

- \`count\`: 1–20, default 5.
- \`interval_ms\`: 100–2000 milliseconds, default 500.

The tool waits one interval before each string and returns a finite sequence. Supply \`_meta.progressToken\` in the MCP call for incremental \`notifications/progress\` over the POST SSE response. Each generated string is JSON in the progress message. The final structured result has a \`results\` array. Without a progress token, only the final JSON response is returned.

Example final result for a count of 2:

\`\`\`json
{"results":[{"name":"divine-limit-4413"},{"name":"wispy-resonance-4355"}]}
\`\`\`

Closing the streaming request stops generation. Aborting a JSON-only call is not guaranteed to stop server-side generation. Calls last at most 40 seconds. Generated strings are not guaranteed to be unique.

## Terms of Use

${site.terms.join("\n\n")}

## Links

- [Website](${site.url}/)
- [API instructions](${site.url}/#api)
- [MCP instructions](${site.url}/#mcp)
- [Terms of Use](${site.url}/#terms)
- [Powered by Haikunator](${site.poweredBy})
- [Source on GitHub](${site.github})
- [Sitemap](${site.url}/sitemap.xml)
`;
writeFileSync(new URL("index.md", destination), markdown);
writeFileSync(new URL("sitemap.xml", destination), `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>${site.url}/</loc></url>
  <url><loc>${site.url}/index.md</loc></url>
</urlset>
`);
writeFileSync(new URL("robots.txt", destination), `User-agent: *\nAllow: /\n\nSitemap: ${site.url}/sitemap.xml\n`);
