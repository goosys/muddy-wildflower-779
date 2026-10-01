# Project terminology

- The product name is **Haikunator Generator**.
- Use the README's opening description as the source of truth: **“A web service that generates Heroku-like memorable random strings.”**
- Describe the generated output as a **string** or **strings**, not a “name”, “random name”, or “random names”. Do not reposition this service as a name generator.
- Use **“Heroku-like memorable random strings”** in the main service description. Short interface labels may use “Generate string”, “Generated string”, “Example string”, and “Copy generated string”.
- Apply this terminology consistently to page copy, button labels, accessible labels, metadata, error messages, documentation, and `/index.md`.
- Preserve existing API/MCP contracts, JSON keys such as `name`, tool names (`gen`, `gen_continuous`), and technical identifiers. This terminology rule does not rename them.

# Site configuration

- Maintain site settings in `frontend/rsbuild.config.ts` and `frontend/src/content/site.json`.
- Keep public URLs, social account identifiers, and shared site content in `site.json`. Read these values from the configuration instead of hardcoding them in implementation or tests.
- Use fictional domains and account identifiers for test fixtures.
- Configure HTML metadata, social cards, assets, and frontend build/development behavior in `frontend/rsbuild.config.ts`, using the shared values from `site.json`.
- When changing deployment URLs or the Worker name, also update `wrangler.toml` as described in `docs/development.md`.

# Generated service-site documents

- `scripts/generate-site-docs.mjs` generates `frontend/public/index.md`, `frontend/public/sitemap.xml`, and `frontend/public/robots.txt` from `frontend/src/content/site.json` and the template in the script.
- Edit the source content or template rather than editing the generated public files directly.
- After editing shared site content or documentation templates, run `node scripts/generate-site-docs.mjs` and include the updated generated public files.
- Frontend build and development commands also run this generator automatically.
