import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [request, setRequest] = useState<unknown>(null);

  useEffect(() => {
    invoke("get_request").then(setRequest);
  }, []);

  return (
    <main className="container">
      <h1>ObjectiveAI Viewer</h1>
      <pre>{request ? JSON.stringify(request, null, 2) : "Loading..."}</pre>
    </main>
  );
}

export default App;
