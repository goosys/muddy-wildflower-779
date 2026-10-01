import site from "../content/site.json";
export function TermsSection() {
  return (
    <section
      id="terms"
      className="terms-section"
      aria-labelledby="terms-heading"
    >
      <h2 id="terms-heading">Terms of Use</h2>
      {site.terms.map((paragraph) => (
        <p key={paragraph}>{paragraph}</p>
      ))}
    </section>
  );
}
