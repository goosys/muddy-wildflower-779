# Project terminology

- The product name is **Haikunator Generator**.
- Use the README's opening description as the source of truth: **“A web service that generates Heroku-like memorable random strings.”**
- Describe the generated output as a **string** or **strings**, not a “name”, “random name”, or “random names”. Do not reposition this service as a name generator.
- Use **“Heroku-like memorable random strings”** in the main service description. Short interface labels may use “Generate string”, “Generated string”, “Example string”, and “Copy generated string”.
- Apply this terminology consistently to page copy, button labels, accessible labels, metadata, error messages, documentation, and `/index.md`.
- Preserve existing API/MCP contracts, JSON keys such as `name`, tool names (`gen`, `gen_continuous`), and technical identifiers. This terminology rule does not rename them.
- After editing shared site content or documentation templates, run `node scripts/generate-site-docs.mjs` and include the updated generated public files.
