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

type Request =
  | FunctionsExecutionsRequestFunctionExecutionCreateParams
  | FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParams;

type Chunk =
  | FunctionsExecutionsResponseStreamingFunctionExecutionChunk
  | FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk
  | ErrorResponseError;

function App() {
  const [request, setRequest] = useState<Request | null>(null);
  const [chunk, setChunk] = useState<Chunk | null>(null);

  useEffect(() => {
    invoke<Request>("get_request").then(setRequest);
    const unlisten = listen<Chunk>("chunk", (event) => {
      setChunk(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <main className="container">
      <h1>ObjectiveAI Viewer</h1>
      <pre>{request ? JSON.stringify(request, null, 2) : "Loading..."}</pre>
      <pre>{chunk ? JSON.stringify(chunk, null, 2) : "Waiting for chunks..."}</pre>
    </main>
  );
}

export default App;
