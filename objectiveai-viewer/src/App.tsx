import { useEffect, useState } from "react";
import cn from "classnames";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { AgentCompletionView } from "./AgentCompletionView";
import { FunctionInventionRecursiveView } from "./FunctionInventionRecursiveView";
import { FunctionExecutionView } from "./FunctionExecutionView";
import { TabBar, type Tab } from "./TabBar";
import { PluginPane } from "./PluginPane";
import { RightOverlayPanel, type PanelTab } from "./RightOverlayPanel";
import { z } from "zod";
import {
  AgentCompletionsRequestAgentCompletionCreateParamsSchema,
  AgentCompletionsResponseStreamingAgentCompletionChunkSchema,
  FunctionsExecutionsRequestFunctionExecutionCreateParamsSchema,
  FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsSchema,
  FunctionsExecutionsResponseStreamingFunctionExecutionChunkSchema,
  FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkSchema,
  LaboratoriesExecutionsRequestLaboratoryExecutionCreateParamsSchema,
  LaboratoriesExecutionsResponseStreamingLaboratoryExecutionChunkSchema,
  ErrorResponseErrorSchema,
  agentCompletionsResponseStreamingAgentCompletionChunkMerged,
  functionsExecutionsResponseStreamingFunctionExecutionChunkMerged,
  functionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkMerged,
  laboratoriesExecutionsResponseStreamingLaboratoryExecutionChunkMerged,
} from "@objectiveai/sdk";
import type {
  AgentCompletionsResponseStreamingAgentCompletionChunk,
  FunctionsExecutionsResponseStreamingFunctionExecutionChunk,
  FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk,
  LaboratoriesExecutionsResponseStreamingLaboratoryExecutionChunk,
} from "@objectiveai/sdk";

// Extended schemas with required id
const AgentCompletionCreateParamsSchema = AgentCompletionsRequestAgentCompletionCreateParamsSchema.extend({
  id: z.string(),
});
type AgentCompletionCreateParams = z.infer<typeof AgentCompletionCreateParamsSchema>;

const FunctionExecutionCreateParamsSchema = FunctionsExecutionsRequestFunctionExecutionCreateParamsSchema.extend({
  id: z.string(),
});
type FunctionExecutionCreateParams = z.infer<typeof FunctionExecutionCreateParamsSchema>;

const FunctionInventionRecursiveCreateParamsSchema = FunctionsInventionsRecursiveRequestFunctionInventionRecursiveCreateParamsSchema.extend({
  id: z.string(),
});
type FunctionInventionRecursiveCreateParams = z.infer<typeof FunctionInventionRecursiveCreateParamsSchema>;

const LaboratoryExecutionCreateParamsSchema = LaboratoriesExecutionsRequestLaboratoryExecutionCreateParamsSchema.extend({
  id: z.string(),
});
type LaboratoryExecutionCreateParams = z.infer<typeof LaboratoryExecutionCreateParamsSchema>;

const ResponseErrorSchema = ErrorResponseErrorSchema.extend({
  id: z.string(),
});
type ResponseError = z.infer<typeof ResponseErrorSchema>;

// Classified incoming event
type AgentCompletionEvent =
  | { type: "begin"; data: AgentCompletionCreateParams }
  | { type: "chunk"; data: AgentCompletionsResponseStreamingAgentCompletionChunk }
  | { type: "error"; data: ResponseError };

type FunctionExecutionEvent =
  | { type: "begin"; data: FunctionExecutionCreateParams }
  | { type: "chunk"; data: FunctionsExecutionsResponseStreamingFunctionExecutionChunk }
  | { type: "error"; data: ResponseError };

type FunctionInventionRecursiveEvent =
  | { type: "begin"; data: FunctionInventionRecursiveCreateParams }
  | { type: "chunk"; data: FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk }
  | { type: "error"; data: ResponseError };

type LaboratoryExecutionEvent =
  | { type: "begin"; data: LaboratoryExecutionCreateParams }
  | { type: "chunk"; data: LaboratoriesExecutionsResponseStreamingLaboratoryExecutionChunk }
  | { type: "error"; data: ResponseError };

// Entry in the list
interface AgentCompletionEntry {
  kind: "agent-completion";
  id: string;
  request: AgentCompletionCreateParams;
  chunk: AgentCompletionsResponseStreamingAgentCompletionChunk | null;
  error: ResponseError | null;
}

interface FunctionExecutionEntry {
  kind: "execution";
  id: string;
  request: FunctionExecutionCreateParams;
  chunk: FunctionsExecutionsResponseStreamingFunctionExecutionChunk | null;
  error: ResponseError | null;
}

interface FunctionInventionRecursiveEntry {
  kind: "invention";
  id: string;
  request: FunctionInventionRecursiveCreateParams;
  chunk: FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk | null;
  error: ResponseError | null;
}

interface LaboratoryExecutionEntry {
  kind: "laboratory";
  id: string;
  request: LaboratoryExecutionCreateParams;
  chunk: LaboratoriesExecutionsResponseStreamingLaboratoryExecutionChunk | null;
  error: ResponseError | null;
}

type Entry = AgentCompletionEntry | FunctionExecutionEntry | FunctionInventionRecursiveEntry | LaboratoryExecutionEntry;

function classifyAgentCompletion(payload: unknown): AgentCompletionEvent | null {
  const beginParse = AgentCompletionCreateParamsSchema.safeParse(payload);
  if (beginParse.success) return { type: "begin", data: beginParse.data };
  const errorParse = ResponseErrorSchema.safeParse(payload);
  if (errorParse.success) return { type: "error", data: errorParse.data };
  const chunkParse = AgentCompletionsResponseStreamingAgentCompletionChunkSchema.safeParse(payload);
  if (chunkParse.success) return { type: "chunk", data: chunkParse.data };
  return null;
}

function classifyFunctionExecution(payload: unknown): FunctionExecutionEvent | null {
  const beginParse = FunctionExecutionCreateParamsSchema.safeParse(payload);
  if (beginParse.success) return { type: "begin", data: beginParse.data };
  const errorParse = ResponseErrorSchema.safeParse(payload);
  if (errorParse.success) return { type: "error", data: errorParse.data };
  const chunkParse = FunctionsExecutionsResponseStreamingFunctionExecutionChunkSchema.safeParse(payload);
  if (chunkParse.success) return { type: "chunk", data: chunkParse.data };
  return null;
}

function classifyFunctionInventionRecursive(payload: unknown): FunctionInventionRecursiveEvent | null {
  const beginParse = FunctionInventionRecursiveCreateParamsSchema.safeParse(payload);
  if (beginParse.success) return { type: "begin", data: beginParse.data };
  const errorParse = ResponseErrorSchema.safeParse(payload);
  if (errorParse.success) return { type: "error", data: errorParse.data };
  const chunkParse = FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkSchema.safeParse(payload);
  if (chunkParse.success) return { type: "chunk", data: chunkParse.data };
  return null;
}

function classifyLaboratoryExecution(payload: unknown): LaboratoryExecutionEvent | null {
  const beginParse = LaboratoryExecutionCreateParamsSchema.safeParse(payload);
  if (beginParse.success) return { type: "begin", data: beginParse.data };
  const errorParse = ResponseErrorSchema.safeParse(payload);
  if (errorParse.success) return { type: "error", data: errorParse.data };
  const chunkParse = LaboratoriesExecutionsResponseStreamingLaboratoryExecutionChunkSchema.safeParse(payload);
  if (chunkParse.success) return { type: "chunk", data: chunkParse.data };
  return null;
}

function EntryView({ entry }: { entry: Entry }) {
  if (entry.kind === "agent-completion") {
    return <AgentCompletionView entry={entry} />;
  }
  if (entry.kind === "invention") {
    return <FunctionInventionRecursiveView entry={entry} />;
  }
  if (entry.kind === "execution") {
    return <FunctionExecutionView entry={entry} />;
  }
  // Laboratory executions keep the raw-JSON fallback for now.
  if (entry.error) {
    return <pre style={{ color: "red" }}>{JSON.stringify(entry.error, null, 2)}</pre>;
  }
  if (entry.chunk) {
    return <pre>{JSON.stringify(entry.chunk, null, 2)}</pre>;
  }
  return <pre style={{ color: "gray" }}>{JSON.stringify(entry.request, null, 2)}</pre>;
}

interface EmittedEvent {
  destination: string;
  type: string;
  value: unknown;
}

function ObjectiveAIView() {
  const [entries, setEntries] = useState<Entry[]>([]);

  useEffect(() => {
    const unlistenObjectiveAI = listen<EmittedEvent>("objectiveai", (event) => {
      const { type, value } = event.payload;

      switch (type) {
        case "agent_completions": {
          const classified = classifyAgentCompletion(value);
          if (!classified) return;
          setEntries((prev) => {
            switch (classified.type) {
              case "begin":
                return [...prev, {
                  kind: "agent-completion" as const,
                  id: classified.data.id,
                  request: classified.data,
                  chunk: null,
                  error: null,
                }];
              case "error": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id)) return prev;
                return prev.map((e) =>
                  e.id === id ? { ...e, error: classified.data } : e
                );
              }
              case "chunk": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id && e.kind === "agent-completion")) return prev;
                return prev.map((e) => {
                  if (e.id !== id || e.kind !== "agent-completion") return e;
                  const [merged] = e.chunk
                    ? agentCompletionsResponseStreamingAgentCompletionChunkMerged(e.chunk, classified.data)
                    : [classified.data, true];
                  return { ...e, chunk: merged };
                });
              }
            }
          });
          return;
        }
        case "functions_executions": {
          const classified = classifyFunctionExecution(value);
          if (!classified) return;
          setEntries((prev) => {
            switch (classified.type) {
              case "begin":
                return [...prev, {
                  kind: "execution",
                  id: classified.data.id,
                  request: classified.data,
                  chunk: null,
                  error: null,
                }];
              case "error": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id)) return prev;
                return prev.map((e) =>
                  e.id === id ? { ...e, error: classified.data } : e
                );
              }
              case "chunk": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id && e.kind === "execution")) return prev;
                return prev.map((e) => {
                  if (e.id !== id || e.kind !== "execution") return e;
                  const [merged] = e.chunk
                    ? functionsExecutionsResponseStreamingFunctionExecutionChunkMerged(e.chunk, classified.data)
                    : [classified.data, true];
                  return { ...e, chunk: merged };
                });
              }
            }
          });
          return;
        }
        case "functions_inventions_recursive": {
          const classified = classifyFunctionInventionRecursive(value);
          if (!classified) return;
          setEntries((prev) => {
            switch (classified.type) {
              case "begin":
                return [...prev, {
                  kind: "invention",
                  id: classified.data.id,
                  request: classified.data,
                  chunk: null,
                  error: null,
                }];
              case "error": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id)) return prev;
                return prev.map((e) =>
                  e.id === id ? { ...e, error: classified.data } : e
                );
              }
              case "chunk": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id && e.kind === "invention")) return prev;
                return prev.map((e) => {
                  if (e.id !== id || e.kind !== "invention") return e;
                  const [merged] = e.chunk
                    ? functionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunkMerged(e.chunk, classified.data)
                    : [classified.data, true];
                  return { ...e, chunk: merged };
                });
              }
            }
          });
          return;
        }
        case "laboratories_executions": {
          const classified = classifyLaboratoryExecution(value);
          if (!classified) return;
          setEntries((prev) => {
            switch (classified.type) {
              case "begin":
                return [...prev, {
                  kind: "laboratory" as const,
                  id: classified.data.id,
                  request: classified.data,
                  chunk: null,
                  error: null,
                }];
              case "error": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id)) return prev;
                return prev.map((e) =>
                  e.id === id ? { ...e, error: classified.data } : e
                );
              }
              case "chunk": {
                const id = classified.data.id;
                if (!prev.some((e) => e.id === id && e.kind === "laboratory")) return prev;
                return prev.map((e) => {
                  if (e.id !== id || e.kind !== "laboratory") return e;
                  const [merged] = e.chunk
                    ? laboratoriesExecutionsResponseStreamingLaboratoryExecutionChunkMerged(e.chunk, classified.data)
                    : [classified.data, true];
                  return { ...e, chunk: merged };
                });
              }
            }
          });
          return;
        }
      }
    });

    // Signal the Rust backend that the listener is registered.
    // Events are buffered on the Rust side until this resolves.
    unlistenObjectiveAI.then(() => invoke("viewer_ready"));

    return () => {
      unlistenObjectiveAI.then((fn) => fn());
    };
  }, []);

  return (
    <main className={cn("mx-auto", "max-w-4xl", "px-4", "py-8", "text-left")}>
      <h1 className={cn("text-2xl", "font-semibold", "mb-4")}>ObjectiveAI Viewer</h1>
      {entries.length === 0 && (
        <p className={cn("text-neutral-500", "dark:text-neutral-400")}>
          Waiting for requests...
        </p>
      )}
      {entries.map((entry) => (
        <EntryView key={entry.id} entry={entry} />
      ))}
    </main>
  );
}

const OBJECTIVEAI_TAB_ID = "objectiveai";

/**
 * Mirror of `objectiveai-viewer/src-tauri/src/plugins.rs::ViewerPluginInfo`.
 * Source of truth is the Rust struct; keep these in sync by hand.
 */
export interface ViewerPluginInfo {
  name: string;
  iframe_src: string;
  mobile_ready: boolean;
}

function App() {
  const [plugins, setPlugins] = useState<ViewerPluginInfo[]>([]);
  const [activeTab, setActiveTab] = useState<string>(OBJECTIVEAI_TAB_ID);

  const [isPanelOpen, setIsPanelOpen] = useState(false);
  const [panelTabs, setPanelTabs] = useState<PanelTab[]>([]);
  const [activePanelTabId, setActivePanelTabId] = useState<string | null>(null);

  useEffect(() => {
    invoke<ViewerPluginInfo[]>("list_plugins_with_viewer")
      .then(setPlugins)
      .catch((e) => {
        // eslint-disable-next-line no-console
        console.warn("list_plugins_with_viewer failed:", e);
      });
  }, []);

  const tabs: Tab[] = [
    { id: OBJECTIVEAI_TAB_ID, label: "ObjectiveAI" },
    ...plugins.map((p) => ({ id: p.name, label: p.name })),
  ];

  const activePlugin = plugins.find((p) => p.name === activeTab);

  return (
    <div className={cn("flex", "flex-col", "h-screen")}>
      <div
        className={cn(
          "flex",
          "flex-row",
          "items-stretch",
          "bg-neutral-100",
          "dark:bg-neutral-900",
          "border-b",
          "border-neutral-300",
          "dark:border-neutral-700",
        )}
      >
        <div className={cn("flex-1", "min-w-0")}>
          <TabBar tabs={tabs} activeTab={activeTab} onSelect={setActiveTab} />
        </div>
        <button
          type="button"
          onClick={() => setIsPanelOpen((v) => !v)}
          aria-label={isPanelOpen ? "Close side panel" : "Open side panel"}
          className={cn(
            "shrink-0",
            "px-3",
            "text-neutral-600",
            "dark:text-neutral-400",
            "hover:text-neutral-900",
            "dark:hover:text-neutral-50",
            "cursor-pointer",
            "text-lg",
          )}
        >
          {isPanelOpen ? "⟩" : "⟨"}
        </button>
      </div>
      <div
        className={cn(
          "relative",
          "flex",
          "flex-col",
          "flex-1",
          "min-h-0",
        )}
      >
        {activeTab === OBJECTIVEAI_TAB_ID ? (
          <ObjectiveAIView />
        ) : activePlugin ? (
          <PluginPane info={activePlugin} />
        ) : null}
        {isPanelOpen && (
          <RightOverlayPanel
            panelTabs={panelTabs}
            setPanelTabs={setPanelTabs}
            activePanelTabId={activePanelTabId}
            setActivePanelTabId={setActivePanelTabId}
          />
        )}
      </div>
    </div>
  );
}

export default App;
