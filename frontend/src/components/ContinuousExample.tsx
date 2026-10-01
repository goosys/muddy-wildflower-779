import { useEffect, useRef, useState } from "react";
import { generateContinuously } from "../lib/mcp";
import { CopyButton } from "./CopyButton";

export function ContinuousExample() {
  const [count, setCount] = useState("5");
  const [sequence, setSequence] = useState<Array<{ id: string; name: string }>>(
    [],
  );
  const [progress, setProgress] = useState(0);
  const [total, setTotal] = useState(5);
  const [status, setStatus] = useState<
    "idle" | "running" | "complete" | "stopped"
  >("idle");
  const [error, setError] = useState("");
  const activeRequest = useRef<AbortController | null>(null);
  const requestedCount = Number(count);
  const validCount =
    Number.isInteger(requestedCount) &&
    requestedCount >= 1 &&
    requestedCount <= 20;

  useEffect(
    () => () => {
      activeRequest.current?.abort();
      activeRequest.current = null;
    },
    [],
  );

  const startSequence = async () => {
    if (!validCount) return;
    activeRequest.current?.abort();
    const controller = new AbortController();
    const sequenceId = crypto.randomUUID();
    activeRequest.current = controller;
    setSequence([]);
    setProgress(0);
    setTotal(requestedCount);
    setError("");
    setStatus("running");
    try {
      const results = await generateContinuously({
        count: requestedCount,
        signal: controller.signal,
        onProgress: (update) => {
          if (activeRequest.current !== controller || controller.signal.aborted)
            return;
          setProgress(update.completed);
          if (
            update.name &&
            Number.isInteger(update.completed) &&
            update.completed > 0
          ) {
            const name = update.name;
            setSequence((previous) => {
              const next = [...previous];
              next[update.completed - 1] = {
                id: `${sequenceId}:${update.completed}`,
                name,
              };
              return next;
            });
          }
        },
      });
      if (activeRequest.current !== controller || controller.signal.aborted)
        return;
      setSequence(
        results.map((result, index) => ({
          id: `${sequenceId}:${index + 1}`,
          name: result.name,
        })),
      );
      setProgress(requestedCount);
      setStatus("complete");
    } catch (failure) {
      if (activeRequest.current !== controller || controller.signal.aborted)
        return;
      setError(
        failure instanceof Error
          ? failure.message
          : "Unable to generate strings. Please try again.",
      );
      setStatus("idle");
    } finally {
      if (activeRequest.current === controller) activeRequest.current = null;
    }
  };

  const stopSequence = () => {
    activeRequest.current?.abort();
    activeRequest.current = null;
    setStatus("stopped");
  };

  return (
    <section className="continuous-example" aria-labelledby="sequence-heading">
      <h4 id="sequence-heading" className="text-lg font-semibold text-gray-800">
        Example: continuous generation
      </h4>
      <p className="text-sm text-gray-600">
        Try the gen_continuous MCP tool: watch strings arrive one at a time,
        then copy your favorite.
      </p>
      <div className="flex flex-wrap items-end gap-3">
        <label
          className="flex flex-col gap-1 text-sm text-gray-700"
          htmlFor="sequence-count"
        >
          How many?
          <input
            id="sequence-count"
            type="number"
            min="1"
            max="20"
            step="1"
            value={count}
            onChange={(event) => setCount(event.target.value)}
            disabled={status === "running"}
            className="w-20 border border-gray-300 rounded-md px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
          />
        </label>
        <button
          type="button"
          onClick={startSequence}
          disabled={status === "running" || !validCount}
          className="px-5 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-300 disabled:cursor-not-allowed"
        >
          Start
        </button>
        {status === "running" && (
          <button
            type="button"
            onClick={stopSequence}
            className="px-5 py-2 border border-gray-300 text-gray-700 rounded-md hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            Stop
          </button>
        )}
      </div>
      <p className="text-xs text-gray-500">
        1–20 strings, one every half second.
      </p>
      {status !== "idle" && (
        <div className="space-y-1">
          <progress
            value={progress}
            max={total}
            aria-label="Generation progress"
            className="w-full h-2 accent-blue-600"
          />
          <p aria-live="polite" className="text-sm text-gray-600">
            {status === "stopped"
              ? "Stopped: "
              : status === "complete"
                ? "Complete: "
                : ""}
            {progress} of {total}
          </p>
        </div>
      )}
      {error && (
        <p role="alert" className="text-sm text-red-700">
          {error}
        </p>
      )}
      {sequence.length > 0 && (
        <ol className="space-y-2" aria-label="Generated strings">
          {sequence.map(({ id, name }, index) => (
            <li
              key={id}
              className="flex items-center gap-3 rounded-md bg-gray-100 px-3 py-2"
            >
              <span className="text-xs text-gray-500">{index + 1}.</span>
              <span className="flex-1 min-w-0 font-medium text-gray-800 break-all">
                {name}
              </span>
              <CopyButton text={name} label={`Copy ${name}`} />
            </li>
          ))}
        </ol>
      )}
    </section>
  );
}
