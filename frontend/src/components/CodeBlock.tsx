import { CopyButton } from "./CopyButton";
export function CodeBlock({
  code,
  label,
  copy = true,
}: {
  code: string;
  label: string;
  copy?: boolean;
}) {
  return (
    <div className="code-example">
      <p className="field-label">{label}</p>
      <div className="code-block">
        <pre>
          <code>{code}</code>
        </pre>
        {copy && <CopyButton text={code} label={`Copy ${label}`} />}
      </div>
    </div>
  );
}
