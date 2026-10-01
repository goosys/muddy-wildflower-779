import { useEffect, useRef, useState } from "react";
import { Icon } from "./Icon";

export function CopyButton({
  text,
  label = "Copy",
  disabled = false,
}: {
  text: string;
  label?: string;
  disabled?: boolean;
}) {
  const [status, setStatus] = useState<
    "idle" | "copying" | "copied" | "failed"
  >("idle");
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const version = useRef(0);
  const currentText = useRef(text);
  useEffect(() => {
    currentText.current = text;
    version.current += 1;
    setStatus("idle");
    return () => {
      version.current += 1;
      if (timer.current) clearTimeout(timer.current);
    };
  }, [text]);
  async function copy() {
    const current = ++version.current;
    if (timer.current) clearTimeout(timer.current);
    setStatus("copying");
    try {
      await navigator.clipboard.writeText(currentText.current);
      if (version.current !== current) return;
      setStatus("copied");
    } catch {
      if (version.current !== current) return;
      setStatus("failed");
    }
    timer.current = setTimeout(() => setStatus("idle"), 2000);
  }
  return (
    <span className="copy-control">
      <button
        type="button"
        className={`copy-button ${status === "copied" ? "is-copied" : ""}`}
        onClick={copy}
        disabled={disabled || status === "copying"}
        aria-label={label}
      >
        <Icon name={status === "copied" ? "check" : "copy"} />
        <span>
          {status === "copied"
            ? "Copied!"
            : status === "failed"
              ? "Retry"
              : "Copy"}
        </span>
      </button>
      <span
        className={status === "failed" ? "copy-error" : "sr-only"}
        role="status"
      >
        {status === "copied"
          ? `${label}: copied to clipboard.`
          : status === "failed"
            ? "Couldn't copy. Please try again."
            : ""}
      </span>
    </span>
  );
}
