import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  FunctionsExecutionsRequestFunctionExecutionCreateParams,
  FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParams,
} from "objectiveai";

type Request =
  | FunctionsExecutionsRequestFunctionExecutionCreateParams
  | FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParams;

function App() {
  const [request, setRequest] = useState<Request | null>(null);

  useEffect(() => {
    invoke<Request>("get_request").then(setRequest);
  }, []);

  return (
    <main className="container">
      <h1>ObjectiveAI Viewer</h1>
      <pre>{request ? JSON.stringify(request, null, 2) : "Loading..."}</pre>
    </main>
  );
}

export default App;
