import site from "../content/site.json";
import { GeneratorWidget } from "./GeneratorWidget";
export function GeneratorSection() {
  return (
    <section className="generator-section" aria-labelledby="generator-heading">
      <p className="eyebrow">Heroku-like memorable random strings</p>
      <h1 id="generator-heading">Haikunator Generator</h1>
      <p className="hero-description">{site.tagline}</p>
      <GeneratorWidget />
    </section>
  );
}
