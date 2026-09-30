"use client";

import { useEffect, useRef, useState } from "react";
import { generateContinuously } from "../lib/mcp";

export const HaikunatorGenerator = () => {
  const [generatedString, setGeneratedString] = useState<string>("");
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

  const generateString = async () => {
    try {
      const response = await fetch("/api/gen");
      if (response.ok) {
        const data: { name: string } = await response.json();
        setGeneratedString(data.name);
      } else {
        throw new Error("Failed to generate string");
      }
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-gray-100 px-4 pt-12 pb-4">
      <header className="mt-auto mb-4 text-right text-gray-500 fixed top-2 right-4">
        <p>
          <a href="https://github.com/goosys/muddy-wildflower-779">Contact</a>
        </p>
      </header>

      <main className="w-full max-w-lg bg-white rounded-lg shadow-md p-8 flex flex-col items-center space-y-6">
        <h1 className="text-2xl font-bold text-gray-800 text-center">
          Haikunator Generator
        </h1>
        <p className="text-gray-600 text-center mt-2">
          Heroku-like memorable random string
        </p>

        <div className="w-full bg-gray-100 p-2 sm:p-4 rounded-md flex items-center">
          <div className="text-center flex-grow pl-4">
            {generatedString ? (
              <p className="text-xl sm:text-2xl font-bold text-center break-all">
                {generatedString}
              </p>
            ) : (
              <p className="text-gray-400 text-xl sm:text-2xl font-bold text-center break-all">
                muddy-wildflower-779
              </p>
            )}
          </div>
          <button
            type="button"
            aria-label="Copy generated string"
            onClick={() => {
              navigator.clipboard.writeText(generatedString);
            }}
            className={`text-gray-600 hover:text-gray-800 focus:outline-none ${generatedString ? "visible" : "invisible"}`}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="32"
              height="32"
              viewBox="0 0 24 24"
            >
              <title>Copy generated string</title>
              <path
                fill="currentColor"
                d="M16 1H4c-1.1 0-2 .9-2 2v14h2V3h12zm3 4H8c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h11c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2m0 16H8V7h11z"
              />
            </svg>
          </button>
        </div>

        <button
          type="button"
          onClick={generateString}
          className="px-6 py-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors duration-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
        >
          Generate
        </button>

        <section
          className="w-full border-t border-gray-200 pt-6 space-y-4"
          aria-labelledby="sequence-heading"
        >
          <h2
            id="sequence-heading"
            className="text-lg font-semibold text-gray-800"
          >
            Generate a sequence
          </h2>
          <p className="text-sm text-gray-600">
            Watch memorable strings appear one at a time, then copy your
            favorite.
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
                  <button
                    type="button"
                    aria-label={`Copy ${name}`}
                    onClick={() => {
                      navigator.clipboard.writeText(name);
                    }}
                    className="text-sm text-blue-700 hover:text-blue-900 focus:outline-none focus:ring-2 focus:ring-blue-500 rounded px-1"
                  >
                    Copy
                  </button>
                </li>
              ))}
            </ol>
          )}
        </section>
      </main>

      <footer className="mt-8 mb-4 text-center text-gray-500">
        <p>
          <a href="https://github.com/nishanths/rust-haikunator">
            Powered by Haikunator
          </a>
        </p>
      </footer>
    </div>
  );
};
