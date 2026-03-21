import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  FunctionsExecutionsRequestFunctionExecutionCreateParams,
  FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParams,
  FunctionsExecutionsResponseStreamingFunctionExecutionChunk,
  FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk,
  ErrorResponseError,
} from "objectiveai";
import {
  functionsExecutionsResponseStreamingFunctionExecutionChunkMerged,
  functionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkMerged,
} from "objectiveai";

type Request =
  | FunctionsExecutionsRequestFunctionExecutionCreateParams
  | FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParams;

type ResponseChunk =
  | FunctionsExecutionsResponseStreamingFunctionExecutionChunk
  | FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk;

type Chunk = ResponseChunk | ErrorResponseError;

function isError(chunk: Chunk): chunk is ErrorResponseError {
  return "code" in chunk && !("object" in chunk);
}

function isExecutionChunk(
  chunk: ResponseChunk,
): chunk is FunctionsExecutionsResponseStreamingFunctionExecutionChunk {
  return (
    chunk.object === "scalar.function.execution.chunk" ||
    chunk.object === "vector.function.execution.chunk"
  );
}

function isInventionChunk(
  chunk: ResponseChunk,
): chunk is FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk {
  return (
    chunk.object === "alpha.scalar.function.invention.recursive.chunk" ||
    chunk.object === "alpha.vector.function.invention.recursive.chunk"
  );
}

function App() {
  const [request, setRequest] = useState<Request | null>(null);
  const [chunk, setChunk] = useState<ResponseChunk | null>(null);
  const [error, setError] = useState<ErrorResponseError | null>(null);

  useEffect(() => {
    invoke<Request>("get_request").then(setRequest);
  }, []);

  useEffect(() => {
    if (!request) return;

    const isExecution = "function" in request && "profile" in request;

    const unlisten = listen<Chunk>("chunk", (event) => {
      const incoming = event.payload;

      if (isError(incoming)) {
        setError(incoming);
        return;
      }

      if (isExecution) {
        if (!isExecutionChunk(incoming)) {
          setError({ code: 500, message: "Expected execution chunk but got invention chunk" });
          return;
        }
        setChunk((prev) => {
          if (!prev || !isExecutionChunk(prev)) return incoming;
          const [merged] = functionsExecutionsResponseStreamingFunctionExecutionChunkMerged(prev, incoming);
          return merged;
        });
      } else {
        if (!isInventionChunk(incoming)) {
          setError({ code: 500, message: "Expected invention chunk but got execution chunk" });
          return;
        }
        setChunk((prev) => {
          if (!prev || !isInventionChunk(prev)) return incoming;
          const [merged] = functionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkMerged(prev, incoming);
          return merged;
        });
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [request]);

  return (
    <main className="container">
      <h1>ObjectiveAI Viewer</h1>
      {error && <pre style={{ color: "red" }}>{JSON.stringify(error, null, 2)}</pre>}
      <pre>{chunk ? JSON.stringify(chunk, null, 2) : "Waiting for chunks..."}</pre>
    </main>
  );
}

export default App;
