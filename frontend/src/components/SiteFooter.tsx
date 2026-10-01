import site from "../content/site.json";
export function SiteFooter() {
  return (
    <footer className="site-footer">
      <p>
        Powered by <a href={site.poweredBy}>Haikunator</a>
      </p>
      <nav aria-label="Resources">
        <a href="/index.md">Markdown</a>
        <a href="/sitemap.xml">Sitemap</a>
        <a href="#terms">Terms of Use</a>
        <a href={site.github}>GitHub</a>
      </nav>
    </footer>
  );
}
