import { ApiSection } from "../components/ApiSection";
import { GeneratorSection } from "../components/GeneratorSection";
import { McpSection } from "../components/McpSection";
import { SiteFooter } from "../components/SiteFooter";
import { SiteHeader } from "../components/SiteHeader";
import { TermsSection } from "../components/TermsSection";

export function HaikunatorGenerator() {
  return (
    <div className="site-shell">
      <a href="#main" className="skip-link">
        Skip to content
      </a>
      <SiteHeader />
      <main id="main">
        <GeneratorSection />
        <section
          className="integrations"
          aria-labelledby="integrations-heading"
        >
          <div className="section-intro">
            <h2 id="integrations-heading">Use it anywhere</h2>
            <p>Generate strings from your terminal or AI tools.</p>
          </div>
          <div className="documentation-grid">
            <ApiSection />
            <McpSection />
          </div>
        </section>
        <TermsSection />
      </main>
      <SiteFooter />
    </div>
  );
}
