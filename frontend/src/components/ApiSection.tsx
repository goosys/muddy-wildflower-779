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
        code={'{"name":"cool-rice-4810"}'}
        copy={false}
      />
      <CodeBlock
        label="Generate a string (plain text)"
        code="GET /api/gen.txt"
        copy={false}
      />
      <CodeBlock
        label="Example response"
        code="divine-limit-4413"
        copy={false}
      />
      <CodeBlock label="Try it with curl" code={`curl ${site.url}/api/gen`} />
    </section>
  );
}
