import site from "../content/site.json";
import { CodeBlock } from "./CodeBlock";
export function ApiSection() {
  return (
    <section
      className="documentation-card"
      id="api"
      aria-labelledby="api-heading"
    >
      <p className="section-kicker">For your applications</p>
      <h3 id="api-heading">REST API</h3>
      <p className="section-description">
        Simple HTTP endpoints. No authentication required.
      </p>
      <CodeBlock
        label="Generate a string (JSON)"
        code="GET /api/gen"
        copy={false}
      />
      <CodeBlock
        label="Example response"
        code={'{"name":"falling-disk-1736"}'}
        copy={false}
      />
      <CodeBlock
        label="Generate a string (plain text)"
        code="GET /api/gen.txt"
        copy={false}
      />
      <CodeBlock
        label="Example response"
        code="broken-wildflower-1928"
        copy={false}
      />
      <CodeBlock label="Try it with curl" code={`curl ${site.url}/api/gen`} />
    </section>
  );
}
