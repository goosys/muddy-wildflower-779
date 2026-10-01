import { useEffect, useRef, useState } from "react";
import { CopyButton } from "./CopyButton";
import { Icon } from "./Icon";

export function GeneratorWidget() {
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [revision, setRevision] = useState(0);
  const request = useRef<AbortController | null>(null);
  useEffect(
    () => () => {
      request.current?.abort();
      request.current = null;
    },
    [],
  );
  async function generate() {
    if (request.current) return;
    const controller = new AbortController();
    request.current = controller;
    const timeout = setTimeout(() => controller.abort(), 15000);
    setLoading(true);
    setError("");
    try {
      const response = await fetch("/api/gen", { signal: controller.signal });
      if (!response.ok) throw new Error("Generation failed");
      const data: unknown = await response.json();
      if (
        !data ||
        typeof data !== "object" ||
        !("name" in data) ||
        typeof data.name !== "string" ||
        !data.name
      )
        throw new Error("Invalid response");
      setName(data.name);
      setRevision((value) => value + 1);
    } catch {
      if (request.current === controller)
        setError("Couldn't generate a string. Please try again.");
    } finally {
      clearTimeout(timeout);
      if (request.current === controller) {
        request.current = null;
        setLoading(false);
      }
    }
  }
  return (
    <div className="generator-widget" aria-busy={loading}>
      <div
        key={revision}
        className={`name-panel ${revision ? "name-updated" : ""}`}
      >
        <p className="field-label">
          {name ? "Generated string" : "Example string"}
        </p>
        <div className="name-row">
          <p className={`generated-name ${name ? "" : "example-string"}`}>
            {name || "muddy-wildflower-779"}
          </p>
          <CopyButton
            text={name}
            label="Copy generated string"
            disabled={!name || loading}
          />
        </div>
      </div>
      <button
        className="primary-button generate-button"
        type="button"
        disabled={loading}
        onClick={generate}
      >
        <Icon name="refresh" className={loading ? "spin" : ""} />
        {loading ? "Generating…" : "Generate string"}
      </button>
      <p className="widget-hint" role="status">
        {loading
          ? "Generating your next string…"
          : name
            ? "A fresh string, ready to use."
            : "A memorable string for your next idea."}
        <span className="sr-only">{name ? ` ${name}` : ""}</span>
      </p>
      {error && (
        <p className="error-message" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
