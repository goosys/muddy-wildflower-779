import site from "../content/site.json";
export function SiteHeader() {
  return (
    <header className="site-header">
      <nav aria-label="Main navigation">
        <a href="#api">API</a>
        <a href="#mcp">MCP</a>
        <a href={site.github}>GitHub</a>
      </nav>
    </header>
  );
}
