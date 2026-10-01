import site from "../content/site.json";
import { CodeBlock } from "./CodeBlock";
import { ContinuousExample } from "./ContinuousExample";
export function McpSection() {
  return (
    <section
      className="documentation-card"
      id="mcp"
      aria-labelledby="mcp-heading"
    >
      <p className="section-kicker">For your AI tools</p>
      <h3 id="mcp-heading">MCP</h3>
      <p className="section-description">
        Connect your AI client with Streamable HTTP. No authentication required.
      </p>
      <CodeBlock label="Server URL" code={`${site.url}/mcp`} />
      <p className="tool-tag">Tool: gen</p>
      <p className="section-description">
        No arguments. Returns a generated string.
      </p>
      <CodeBlock
        label="Example response"
        code={'{"name":"cool-rice-4810"}'}
        copy={false}
      />
      <p className="tool-tag">Tool: gen_continuous</p>
      <p className="example-details">
        <code>gen_continuous</code> accepts <code>count</code> (1–20, default 5)
        and <code>interval_ms</code> (100–2000 ms, default 500). Supply a
        progress token for incremental results; otherwise, you receive the final
        result only.
      </p>
      <CodeBlock
        label="Example tool arguments"
        code={'{"count":5,"interval_ms":500}'}
      />
      <CodeBlock
        label="Final result (count: 1 example)"
        code={'{"results":[{"name":"divine-limit-4413"}]}'}
        copy={false}
      />
      <ContinuousExample />
    </section>
  );
}
