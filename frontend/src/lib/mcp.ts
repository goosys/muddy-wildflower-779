export type GeneratedString = { name: string };

export type GenerationProgress = {
  completed: number;
  total: number;
  name?: string;
};

type JsonObject = Record<string, unknown>;

const isObject = (value: unknown): value is JsonObject =>
  typeof value === "object" && value !== null && !Array.isArray(value);

const errorMessage = (value: unknown): string =>
  isObject(value) && typeof value.message === "string"
    ? value.message
    : "Unable to generate strings. Please try again.";

async function readReply(
  response: Response,
  id: string,
  signal: AbortSignal,
  onNotification?: (message: JsonObject) => void,
): Promise<JsonObject> {
  if (!response.ok) {
    throw new Error("Unable to generate strings. Please try again.");
  }

  let reply: JsonObject | undefined;
  const receive = (value: unknown) => {
    signal.throwIfAborted();
    if (!isObject(value)) {
      throw new Error("The response could not be read. Please try again.");
    }
    if (value.id === id) {
      if (value.error !== undefined) {
        throw new Error(errorMessage(value.error));
      }
      if (!isObject(value.result)) {
        throw new Error("The response could not be read. Please try again.");
      }
      reply = value.result;
    } else if (typeof value.method === "string") {
      onNotification?.(value);
    }
  };

  if (!response.headers.get("content-type")?.includes("text/event-stream")) {
    receive(await response.json());
  } else {
    if (!response.body) {
      throw new Error("The response ended early. Please try again.");
    }

    const reader = response.body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    let data: string[] = [];

    const dispatch = () => {
      if (data.length === 0) return;
      const payload = data.join("\n");
      data = [];
      if (!payload) return;
      receive(JSON.parse(payload));
    };
    const line = (value: string) => {
      if (value === "") {
        dispatch();
      } else if (value.startsWith("data:")) {
        data.push(value.slice(5).replace(/^ /, ""));
      } else if (value === "data") {
        data.push("");
      }
    };
    const consume = (ended: boolean) => {
      while (!reply) {
        const separator = buffer.search(/[\r\n]/);
        if (separator < 0) break;
        if (
          !ended &&
          buffer[separator] === "\r" &&
          separator === buffer.length - 1
        ) {
          break;
        }
        const length =
          buffer[separator] === "\r" && buffer[separator + 1] === "\n" ? 2 : 1;
        const value = buffer.slice(0, separator);
        buffer = buffer.slice(separator + length);
        line(value);
      }
      if (ended && !reply) {
        if (buffer) line(buffer);
        dispatch();
      }
    };

    try {
      while (!reply) {
        signal.throwIfAborted();
        const chunk = await reader.read();
        buffer += decoder.decode(chunk.value, { stream: !chunk.done });
        consume(chunk.done);
        if (chunk.done) break;
      }
    } finally {
      await reader.cancel().catch(() => {});
      reader.releaseLock();
    }
  }

  signal.throwIfAborted();
  if (!reply) {
    throw new Error("The response ended early. Please try again.");
  }
  return reply;
}

export async function generateContinuously({
  count,
  signal,
  onProgress,
}: {
  count: number;
  signal: AbortSignal;
  onProgress: (progress: GenerationProgress) => void;
}): Promise<GeneratedString[]> {
  const token = crypto.randomUUID();
  const headers = {
    "Content-Type": "application/json",
    Accept: "application/json, text/event-stream",
    "MCP-Protocol-Version": "2025-11-25",
  };
  const post = (message: JsonObject) =>
    fetch("/mcp", {
      method: "POST",
      headers,
      body: JSON.stringify({ jsonrpc: "2.0", ...message }),
      signal,
    });

  const initializeId = `${token}:initialize`;
  await readReply(
    await post({
      id: initializeId,
      method: "initialize",
      params: {
        protocolVersion: "2025-11-25",
        capabilities: {},
        clientInfo: { name: "haikunator-generator-web", version: "0.1.0" },
      },
    }),
    initializeId,
    signal,
  );
  const initialized = await post({ method: "notifications/initialized" });
  if (!initialized.ok) {
    throw new Error("Unable to generate strings. Please try again.");
  }

  const callId = `${token}:call`;
  const result = await readReply(
    await post({
      id: callId,
      method: "tools/call",
      params: {
        name: "gen_continuous",
        arguments: { count, interval_ms: 500 },
        _meta: { progressToken: token },
      },
    }),
    callId,
    signal,
    (notification) => {
      if (
        notification.method !== "notifications/progress" ||
        !isObject(notification.params) ||
        notification.params.progressToken !== token
      ) {
        return;
      }
      const params = notification.params;
      if (
        typeof params.progress !== "number" ||
        !Number.isFinite(params.progress)
      ) {
        return;
      }
      let name: string | undefined;
      if (typeof params.message === "string") {
        try {
          const item: unknown = JSON.parse(params.message);
          if (isObject(item) && typeof item.name === "string") name = item.name;
        } catch {
          // An initial progress update may contain a human-readable message.
        }
      }
      onProgress({
        completed: Math.max(0, Math.min(count, params.progress)),
        total: count,
        name,
      });
    },
  );

  if (result.isError === true) {
    throw new Error("Unable to generate strings. Please try again.");
  }
  let content: unknown = result.structuredContent;
  if (!isObject(content) && Array.isArray(result.content)) {
    const text = result.content.find(
      (item: unknown) =>
        isObject(item) && item.type === "text" && typeof item.text === "string",
    );
    if (isObject(text) && typeof text.text === "string")
      content = JSON.parse(text.text);
  }
  if (
    !isObject(content) ||
    !Array.isArray(content.results) ||
    content.results.length !== count ||
    !content.results.every(
      (item: unknown) =>
        isObject(item) && typeof item.name === "string" && item.name.length > 0,
    )
  ) {
    throw new Error("The response could not be read. Please try again.");
  }
  return content.results as GeneratedString[];
}
