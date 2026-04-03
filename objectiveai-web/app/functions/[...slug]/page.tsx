"use client";

import { useState, useEffect, useCallback, use } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { createPublicClient } from "../../../lib/client";
import { deriveDisplayName } from "../../../lib/objectiveai";
import { PINNED_COLOR_ANIMATION_MS } from "../../../lib/constants";
import { DEFAULT_PROFILES } from "../../../lib/profiles";
import { loadReasoningModels } from "../../../lib/reasoning-models";
import { useIsMobile } from "../../../hooks/useIsMobile";
import { useObjectiveAI } from "../../../hooks/useObjectiveAI";
import SplitItemDisplay from "../../../components/SplitItemDisplay";
import { simplifySplitItems, toDisplayItem, getDisplayMode } from "../../../lib/split-item-utils";
import { compileFunctionInputSplit, type FunctionConfig } from "../../../lib/wasm-validation";
import type { InputValue } from "../../../components/SchemaForm/types";
import { Functions, EnsembleLlm } from "objectiveai";
import { SkeletonFunctionDetails } from "../../../components/ui";
import { FunctionTree } from "@objectiveai/function-tree";
import type { InputFunctionDefinition } from "@objectiveai/function-tree";
import "@objectiveai/function-tree/styles";
import { useResolvedSubFunctions } from "../../../hooks/useResolvedSubFunctions";
import { ChatBar } from "../../../components/ChatBar/ChatBar";
import { useChatOrchestration } from "../../../components/ChatBar/useChatOrchestration";
interface FunctionDetails {
  owner: string;
  repository: string;
  commit: string;
  name: string;
  description: string;
  category: string;
  type: "scalar.function" | "vector.function";
  inputSchema: Record<string, unknown> | null;
}

export default function FunctionDetailPage({ params }: { params: Promise<{ slug: string[] }> }) {
  const { slug } = use(params);

  const router = useRouter();

  // Parse slug: catch-all route gives us string[] e.g. ["owner", "repo"]
  const owner = slug[0] || "unknown";
  const repository = slug.length >= 2 ? slug[1] : slug[0] || "unknown";

  // Backward compat: old "--" URLs redirect to new "/" format
  useEffect(() => {
    if (slug.length === 1 && slug[0].includes("--")) {
      router.replace(`/functions/${slug[0].replace("--", "/")}`);
    }
  }, [slug, router]);

  // Canonical key for localStorage pinning (owner/repo format)
  const slugKey = `${owner}/${repository}`;

  const [functionDetails, setFunctionDetails] = useState<FunctionDetails | null>(null);
  const [selectedProfileIndex, setSelectedProfileIndex] = useState(0);
  const [availableProfiles, setAvailableProfiles] = useState<Array<{
    owner: string;
    repository: string;
    commit: string | null;
    label: string;
    description: string;
  }>>(DEFAULT_PROFILES);
  const [isLoadingDetails, setIsLoadingDetails] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [isRunning, setIsRunning] = useState(false);
  const isMobile = useIsMobile();
  const { getClient } = useObjectiveAI();
  const [isSaved, setIsSaved] = useState(false);
  const [showPinnedColor, setShowPinnedColor] = useState(false);
  const [splitItems, setSplitItems] = useState<InputValue[] | null>(null);
  const [results, setResults] = useState<{
    output?: number | number[];
    inputSnapshot?: Record<string, unknown>; // Store input for display
    usage?: {
      prompt_tokens: number;
      completion_tokens: number;
      total_tokens: number;
      cost?: number;
      total_cost?: number;
    };
    tasks?: Array<{
      votes?: Array<{
        model: string;
        vote: number[];
        weight: number;
        from_cache?: boolean;
        from_rng?: boolean;
      }>;
      completions?: Array<{
        model: string;
        choices?: Array<{
          message?: {
            content?: string;
          };
          delta?: {
            content?: string;
          };
        }>;
      }>;
      scores?: number[];
    }>;
    reasoning?: {
      choices?: Array<{
        message?: {
          content?: string;
        };
      }>;
    } | null;
    error?: string;
  } | null>(null);
  const [modelNames, setModelNames] = useState<Record<string, string>>({});
  const [showAllModels, setShowAllModels] = useState(false);
  const [expandedVotes, setExpandedVotes] = useState<Set<number>>(new Set());
  const [rawDefinition, setRawDefinition] = useState<InputFunctionDefinition | null>(null);
  const [showDetailedResults, setShowDetailedResults] = useState(false);

  // Demo mode: when enabled, uses RNG votes (free, simulated). When disabled, uses real LLM inference.
  const [demoMode, setDemoMode] = useState(true);

  // Reasoning options
  const [reasoningEnabled, setReasoningEnabled] = useState(false);
  const [reasoningModel, setReasoningModel] = useState(""); // Set after loading from JSON
  const [reasoningModels, setReasoningModels] = useState<Array<{ value: string; label: string }>>([]);

  // Fetch function details
  useEffect(() => {
    // Skip fetching if this is a legacy "--" URL that will redirect
    if (slug.length === 1 && slug[0].includes("--")) return;

    async function fetchDetails() {
      try {
        setIsLoadingDetails(true);
        setLoadError(null);

        const publicClient = createPublicClient();

        // Fetch function details directly (works for all functions, regardless of profiles)
        const details = await Functions.retrieve(publicClient, "github", owner, repository, null);

        const category = details.type === "vector.function" ? "Ranking" : "Scoring";

        setFunctionDetails({
          owner,
          repository,
          commit: details.commit || "",
          name: deriveDisplayName(repository),
          description: details.description || `${deriveDisplayName(repository)} function`,
          category,
          type: details.type as "scalar.function" | "vector.function",
          inputSchema: (details as { input_schema?: Record<string, unknown> }).input_schema || null,
        });

        // Store raw definition for structural tree
        setRawDefinition(details as unknown as InputFunctionDefinition);

        // Only show default profiles (Nano, Mini, Standard, Giga, Giga Max)
        setAvailableProfiles(DEFAULT_PROFILES);
        setSelectedProfileIndex(0);
      } catch (err) {
        setLoadError(err instanceof Error ? err.message : "Failed to load function");
      } finally {
        setIsLoadingDetails(false);
      }
    }

    fetchDetails();
  }, [owner, repository]);

  // Load saved state from localStorage + migrate old "--" keys
  useEffect(() => {
    const savedLibrary = localStorage.getItem("pinned-functions");
    if (savedLibrary) {
      const library: string[] = JSON.parse(savedLibrary);
      // Migrate old "--" format keys to "/" format
      const migrated = library.map((s: string) => s.includes("--") ? s.replace("--", "/") : s);
      if (JSON.stringify(migrated) !== JSON.stringify(library)) {
        localStorage.setItem("pinned-functions", JSON.stringify(migrated));
      }
      setIsSaved(migrated.includes(slugKey));
    }
  }, [slugKey]);

  // Load reasoning models from build-time generated JSON
  useEffect(() => {
    loadReasoningModels().then(config => {
      setReasoningModels(config.models.map(m => ({ value: m.value, label: m.label })));
      setReasoningModel(config.default_model);
    });
  }, []);

  // Resolve sub-function definitions for structural tree
  const resolvedSubFunctions = useResolvedSubFunctions(rawDefinition);

  // Chat orchestration — replaces old manual input form
  const selectedProfile = availableProfiles[selectedProfileIndex];

  const handleExecutionUpdate = useCallback((chunk: unknown) => {
    // Merge streaming chunk into results (same logic as old handleRun)
    const c = chunk as Record<string, unknown>;

    setResults(prev => {
      const existing = prev || {};
      const output = c.output !== undefined ? c.output as number | number[] : existing.output;
      const usage = c.usage ? c.usage as NonNullable<typeof results>["usage"] : existing.usage;

      // Merge tasks
      let tasks = existing.tasks;
      if (c.tasks && Array.isArray(c.tasks)) {
        const incoming = c.tasks as NonNullable<typeof results>["tasks"];
        if (!tasks || tasks.length === 0) {
          tasks = incoming;
        } else {
          const merged = [...tasks];
          for (const task of incoming!) {
            if (!task) continue;
            const taskIndex = (task as { index?: number }).index;
            const existingIdx = merged.findIndex(t => t && (t as { index?: number }).index === taskIndex);
            if (existingIdx === -1) {
              merged.push(task);
            } else {
              merged[existingIdx] = { ...merged[existingIdx], ...task };
            }
          }
          tasks = merged;
        }
      }

      // Merge reasoning
      let reasoning = existing.reasoning;
      const rc = c.reasoning as { choices?: Array<{ delta?: { content?: string }; message?: { content?: string } }> } | undefined;
      if (rc?.choices?.[0]?.delta?.content) {
        const prev = reasoning?.choices?.[0]?.message?.content || "";
        reasoning = { choices: [{ message: { content: prev + rc.choices[0].delta.content } }] };
      } else if (rc?.choices?.[0]?.message?.content) {
        reasoning = { choices: [{ message: { content: rc.choices[0].message.content } }] };
      }

      return { ...existing, output, usage, tasks, reasoning } as typeof results;
    });
  }, []);

  const handleExecutionStart = useCallback(() => {
    setIsRunning(true);
    setResults(null);
    setSplitItems(null);
    setShowAllModels(false);
    setExpandedVotes(new Set());
  }, []);

  const handleExecutionEnd = useCallback(() => {
    setIsRunning(false);
  }, []);

  const { messages, chatState, sendMessage, clearMessages } = useChatOrchestration({
    functionMeta: functionDetails,
    profile: selectedProfile ? {
      owner: selectedProfile.owner,
      repository: selectedProfile.repository,
      commit: selectedProfile.commit,
    } : { owner: "ObjectiveAI", repository: "profile-nano", commit: null },
    demoMode,
    reasoningEnabled,
    reasoningModel,
    onExecutionUpdate: handleExecutionUpdate,
    onExecutionStart: handleExecutionStart,
    onExecutionEnd: handleExecutionEnd,
  });

  // Toggle save state
  const toggleSave = () => {
    const savedLibrary = localStorage.getItem("pinned-functions");
    const library = savedLibrary ? JSON.parse(savedLibrary) : [];

    if (isSaved) {
      const updated = library.filter((s: string) => s !== slugKey);
      localStorage.setItem("pinned-functions", JSON.stringify(updated));
      setIsSaved(false);
    } else {
      library.push(slugKey);
      localStorage.setItem("pinned-functions", JSON.stringify(library));
      setIsSaved(true);
      setShowPinnedColor(true);
      setTimeout(() => setShowPinnedColor(false), PINNED_COLOR_ANIMATION_MS);
    }
  };

  // Fetch model names when results contain votes using SDK
  useEffect(() => {
    if (!results?.tasks || !Array.isArray(results.tasks) || results.tasks.length === 0) return;

    const allVotes = results.tasks.flatMap(t => (t && t.votes) ? t.votes : []);
    if (allVotes.length === 0) return;

    const uniqueIds = [...new Set(allVotes.filter(v => v?.model).map(v => v.model))];
    const idsToFetch = uniqueIds.filter(id => id && !modelNames[id]);

    if (idsToFetch.length === 0) return;

    // Fetch in parallel using SDK
    (async () => {
      const client = await getClient();
      const fetchResults = await Promise.all(
        idsToFetch.map(async (id) => {
          try {
            const llm = await EnsembleLlm.retrieve(client, id);
            return { id, model: llm.model as string };
          } catch {
            // Ignore errors, fall back to cryptic ID
            return null;
          }
        })
      );
      const newNames: Record<string, string> = {};
      for (const r of fetchResults) {
        if (r) newNames[r.id] = r.model;
      }
      if (Object.keys(newNames).length > 0) {
        setModelNames(prev => ({ ...prev, ...newNames }));
      }
    })();
  }, [results?.tasks, modelNames, getClient]);

  // Compute split items for vector results visualization
  useEffect(() => {
    if (!results?.output || !Array.isArray(results.output) || !functionDetails) return;

    // Capture values for the async function
    const inputSnapshot = results.inputSnapshot;
    const { owner, repository, commit } = functionDetails;

    async function computeSplitItems() {
      try {
        // Fetch the full function definition for WASM compilation
        const publicClient = createPublicClient();
        const funcDef = await Functions.retrieve(publicClient, "github", owner, repository, commit);

        // Use WASM to compile the input split
        const splitResult = await compileFunctionInputSplit(funcDef as unknown as FunctionConfig, inputSnapshot);
        if (splitResult.success && splitResult.data) {
          // Simplify the items for display (cast to InputValue[])
          const simplified = simplifySplitItems(splitResult.data as InputValue[]);
          setSplitItems(simplified);
        }
      } catch {
        // Keep splitItems as null, will fall back to basic labels
      }
    }

    computeSplitItems();
  }, [results?.output, results?.inputSnapshot, functionDetails]);

  // Score color gradient: green (100%) → yellow (66%) → orange (33%) → red (0%)
  const getScoreColor = (percentage: number): string => {
    if (percentage >= 66) return "var(--color-success)"; // green
    if (percentage >= 33) return "var(--color-warning)"; // yellow
    if (percentage >= 15) return "var(--color-danger)";  // orange
    return "var(--color-error)";                          // red
  };

  // Helper to get content item label
  const getContentLabel = (index: number): string => {
    const letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
    const input = results?.inputSnapshot;

    // Try to get actual content from input
    const contentItems = input?.contentItems as unknown[] | undefined;
    if (contentItems && contentItems[index] !== undefined) {
      const item = contentItems[index];
      if (typeof item === "string") {
        // RichContent::Text - plain string
        return item.length > 40 ? item.slice(0, 40) + "..." : item;
      }
      // RichContent::Parts - array of RichContentPart
      // See: objectiveai-rs/src/chat/completions/request/message.rs
      if (Array.isArray(item) && item.length > 0) {
        const part = item[0] as { type?: string; file?: { filename?: string } };
        if (part?.type === "file" && part?.file?.filename) {
          return part.file.filename;
        }
        if (part?.type === "image_url") return "[Image]";
        if (part?.type === "input_audio") return "[Audio]";
        if (part?.type === "video_url") return "[Video]";
        return "[Media content]";
      }
    }

    return `Option ${letters[index] || index + 1}`;
  };

  // Render results based on output type
  const renderResults = () => {
    if (!results?.output) return null;

    const output = results.output;

    // Scalar output (single number)
    if (typeof output === "number") {
      const pct = output * 100;
      const keywords = results.inputSnapshot?.keywords as string[] | undefined;
      const scoreColor = getScoreColor(pct);

      return (
        <div>
          <p style={{
            fontSize: "13px",
            color: "var(--text-muted)",
            marginBottom: "6px",
          }}>
            Overall Score
          </p>
          <p style={{
            fontSize: isMobile ? "42px" : "56px",
            fontWeight: 700,
            color: scoreColor,
            lineHeight: 1,
            marginBottom: "12px",
          }}>
            {pct.toFixed(1)}%
          </p>
          {/* Score bar */}
          <div style={{
            height: "10px",
            background: "var(--border)",
            borderRadius: "5px",
            overflow: "hidden",
            marginBottom: "16px",
          }}>
            <div style={{
              height: "100%",
              width: `${pct}%`,
              background: scoreColor,
              borderRadius: "5px",
              transition: "width 0.5s ease",
            }} />
          </div>
          {keywords && keywords.length > 0 && (
            <p style={{ fontSize: "13px", color: "var(--text-muted)" }}>
              Relevance to: <span style={{ color: "var(--text)" }}>{keywords.join(", ")}</span>
            </p>
          )}
        </div>
      );
    }

    // Vector output (array of numbers) - Rankings
    if (Array.isArray(output)) {
      const sorted = output
        .map((score, i) => ({ index: i, score, label: getContentLabel(i) }))
        .sort((a, b) => b.score - a.score);

      const keywords = results.inputSnapshot?.keywords as string[] | undefined;

      // Determine display mode based on split items
      const displayMode = splitItems ? getDisplayMode(splitItems) : "simple";
      const showCompactDisplay = displayMode === "simple" || displayMode === "mixed";

      return (
        <div>
          {/* Show keywords context */}
          {keywords && keywords.length > 0 && (
            <p style={{
              fontSize: "13px",
              color: "var(--text-muted)",
              marginBottom: "16px",
            }}>
              Ranked by relevance to: <span style={{ color: "var(--text)" }}>{keywords.join(", ")}</span>
            </p>
          )}

          <p style={{
            fontSize: "13px",
            color: "var(--text-muted)",
            marginBottom: "12px",
          }}>
            Rankings
          </p>

          <div style={{ display: "flex", flexDirection: "column", gap: isMobile ? "6px" : "8px" }}>
            {sorted.map((item, rank) => {
              const pct = item.score * 100;
              const isTop = rank === 0;
              const splitItem = splitItems?.[item.index];

              return (
                <div key={item.index} style={{
                  display: "flex",
                  alignItems: showCompactDisplay ? "center" : "flex-start",
                  gap: isMobile ? "10px" : "14px",
                  padding: isMobile ? "10px 12px" : "14px 18px",
                  background: isTop ? "rgba(34, 197, 94, 0.08)" : "var(--page-bg)",
                  borderRadius: "6px",
                  border: isTop ? "1px solid rgba(34, 197, 94, 0.2)" : "1px solid transparent",
                }}>
                  <span style={{
                    fontSize: isMobile ? "14px" : "16px",
                    fontWeight: 700,
                    color: getScoreColor(pct),
                    width: isMobile ? "42px" : "50px",
                    flexShrink: 0,
                  }}>
                    {pct.toFixed(0)}%
                  </span>
                  <div style={{
                    flex: 1,
                    fontSize: isMobile ? "13px" : "14px",
                    fontWeight: isTop ? 600 : 400,
                    color: isTop ? "var(--text)" : "var(--text-muted)",
                    overflow: "hidden",
                    minWidth: 0,
                  }}>
                    {splitItem !== undefined ? (
                      <SplitItemDisplay
                        item={toDisplayItem(splitItem)}
                        compact={showCompactDisplay}
                      />
                    ) : (
                      <span style={{
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                        display: "block",
                      }}>
                        {item.label}
                      </span>
                    )}
                  </div>
                  {isTop && !isMobile && (
                    <span style={{
                      fontSize: "11px",
                      padding: "3px 8px",
                      background: "rgba(34, 197, 94, 0.15)",
                      color: "var(--color-success)",
                      borderRadius: "6px",
                      fontWeight: 600,
                      flexShrink: 0,
                    }}>
                      Best Match
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      );
    }

    return null;
  };

  // Loading state
  if (isLoadingDetails) {
    return <SkeletonFunctionDetails />;
  }

  // Error state
  if (loadError || !functionDetails) {
    return (
      <div className="page">
        <div className="container" style={{ paddingTop: "100px", textAlign: "center" }}>
          <p style={{ color: "var(--color-error)", marginBottom: "8px" }}>Failed to load function</p>
          <p style={{ color: "var(--text-muted)", marginBottom: "24px" }}>{loadError}</p>
          <Link href="/functions" style={{ color: "var(--accent)" }}>
            Back to Functions
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="page" style={{ padding: 0, height: "calc(100dvh - var(--nav-height-actual, 64px))", overflow: "hidden", display: "flex", flexDirection: "column" }}>
      {/* Tree viewport — fills remaining space */}
      <div style={{
        position: "relative",
        flex: 1,
        minHeight: 0,
        overflow: "hidden",
      }}>
          <FunctionTree
            data={results ? {
              output: results.output,
              tasks: results.tasks as any,
              function: functionDetails ? `${functionDetails.owner}/${functionDetails.repository}` : undefined,
              reasoning: results.reasoning,
            } : null}
            definition={rawDefinition}
            resolvedSubFunctions={resolvedSubFunctions}
            modelNames={modelNames}
            height="100%"
            borderless
            config={{ theme: "auto", transparentBg: true }}
          />

          {/* Floating header card — top-left */}
          <div style={{
            position: "absolute",
            top: isMobile ? 12 : 16,
            left: isMobile ? 12 : 20,
            right: isMobile ? 12 : undefined,
            maxWidth: isMobile ? undefined : "600px",
            zIndex: 10,
            background: "var(--card-bg)",
            borderRadius: "6px",
            border: "1px solid var(--border)",
            backdropFilter: "blur(16px)",
            boxShadow: "0 2px 8px rgba(0,0,0,0.06)",
            overflow: "hidden",
          }}>
            {/* Top row: breadcrumb, name, tags, pin */}
            <div style={{
              display: "flex",
              alignItems: "center",
              gap: "10px",
              padding: isMobile ? "8px 12px" : "8px 16px",
            }}>
              <nav style={{
                display: "flex",
                gap: "6px",
                color: "var(--text-muted)",
                fontSize: "13px",
                flexShrink: 0,
              }}>
                <Link href="/functions" style={{ color: "var(--accent)", textDecoration: "none" }}>
                  Functions
                </Link>
                <span>/</span>
              </nav>
              <h1 style={{
                fontSize: isMobile ? "15px" : "16px",
                fontWeight: 700,
                margin: 0,
                whiteSpace: "nowrap",
                overflow: "hidden",
                textOverflow: "ellipsis",
                minWidth: 0,
              }}>
                {functionDetails.name}
              </h1>
              {!isMobile && (
                <div style={{ display: "flex", gap: "6px", flexShrink: 0 }}>
                  <span className="tag" style={{ display: "inline-block", fontSize: "11px", padding: "2px 8px" }}>{functionDetails.category}</span>
                  <span style={{
                    fontSize: "11px",
                    padding: "2px 8px",
                    background: "var(--border)",
                    borderRadius: "4px",
                    color: "var(--text-muted)",
                  }}>
                    {functionDetails.owner}/{functionDetails.repository}
                  </span>
                </div>
              )}
              <button
                onClick={toggleSave}
                style={{
                  background: "none",
                  border: "none",
                  padding: 0,
                  cursor: "pointer",
                  fontSize: "13px",
                  color: showPinnedColor ? "var(--accent)" : "var(--text-muted)",
                  opacity: 0.7,
                  transition: showPinnedColor ? "color 0.15s ease-in" : "color 0.5s ease-out",
                  flexShrink: 0,
                  marginLeft: "auto",
                }}
              >
                {isSaved ? "Pinned" : "Pin"}
              </button>
            </div>
            {/* Description row */}
            {!isMobile && functionDetails.description && (
              <p style={{
                fontSize: "11px",
                color: "var(--text-muted)",
                margin: 0,
                padding: "0 16px 8px",
                lineHeight: 1.3,
                opacity: 0.7,
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}>
                {functionDetails.description}
              </p>
            )}
          </div>

      </div>

      {/* Chat bar — bottom of flex layout */}
      <div style={{ flex: "0 0 auto", padding: isMobile ? "8px 0 12px" : "12px 0 16px" }}>
        <ChatBar
          messages={messages}
          chatState={chatState}
          onSend={sendMessage}
          onClear={clearMessages}
          profiles={availableProfiles}
          selectedProfileIndex={selectedProfileIndex}
          onProfileChange={setSelectedProfileIndex}
          demoMode={demoMode}
          onDemoModeChange={setDemoMode}
          reasoningEnabled={reasoningEnabled}
          onReasoningChange={setReasoningEnabled}
          isMobile={isMobile}
          isExecuting={isRunning}
        />
      </div>

      {/* Detailed Results — below the canvas, in a container */}
      {results && !isRunning && (
        <div className="container" style={{ paddingTop: isMobile ? "16px" : "24px", paddingBottom: "32px" }}>
          <button
            onClick={() => setShowDetailedResults((v) => !v)}
            style={{
              background: "none",
              border: "none",
              padding: "8px 0",
              cursor: "pointer",
              fontSize: "13px",
              color: "var(--text-muted)",
              display: "flex",
              alignItems: "center",
              gap: "6px",
            }}
          >
            <span style={{
              transform: showDetailedResults ? "rotate(90deg)" : "rotate(0deg)",
              transition: "transform 0.15s ease",
              display: "inline-block",
              fontSize: "10px",
            }}>
              ▶
            </span>
            Detailed Results
          </button>
            {showDetailedResults && (
              <div className="card" style={{ padding: isMobile ? "12px" : "16px", marginTop: "8px" }}>
                {/* Model Breakdown */}
                {results.tasks && Array.isArray(results.tasks) && results.tasks.length > 0 && results.tasks[0]?.votes && results.tasks[0].votes.length > 0 && (
                  <div>
                    {(() => {
                      const votes = results.tasks![0].votes!;
                      const allSimulated = votes.every(v => v.from_rng);
                      const letters = ["A", "B", "C", "D", "E", "F", "G", "H"];

                      // Get content labels from split items or fallback to input
                      const getOptionLabel = (idx: number): string => {
                        // Use split items if available (simplified, actual content)
                        if (splitItems && splitItems[idx] !== undefined) {
                          const item = splitItems[idx];
                          if (typeof item === "string") {
                            return item.length > 18 ? item.slice(0, 18) + "…" : item;
                          }
                          if (typeof item === "number" || typeof item === "boolean") {
                            return String(item);
                          }
                          // For complex items, show a brief summary
                          const display = toDisplayItem(item);
                          if (display.type === "image") return "[Image]";
                          if (display.type === "audio") return "[Audio]";
                          if (display.type === "video") return "[Video]";
                          if (display.type === "file") return display.filename || "[File]";
                          if (display.type === "object" || display.type === "array") {
                            const json = JSON.stringify(item);
                            return json.length > 18 ? json.slice(0, 18) + "…" : json;
                          }
                          return String(item);
                        }
                        // Fallback to old behavior
                        const contentItems = results.inputSnapshot?.contentItems as unknown[] | undefined;
                        if (contentItems && contentItems[idx]) {
                          const item = contentItems[idx];
                          if (typeof item === "string") {
                            return item.length > 18 ? item.slice(0, 18) + "…" : item;
                          }
                        }
                        return `Option ${letters[idx] || idx + 1}`;
                      };

                      return (
                        <>
                          <p style={{
                            fontSize: isMobile ? "12px" : "13px",
                            color: "var(--text-muted)",
                            marginBottom: isMobile ? "12px" : "16px",
                          }}>
                            Model Breakdown
                          </p>

                          <div className="model-breakdown-wrapper">
                            <div style={{ display: "flex", flexDirection: "column", gap: isMobile ? "12px" : "16px" }}>
                            {(() => {
                              const displayedVotes = showAllModels ? votes : votes.slice(0, 5);
                              const completions = results.tasks?.[0]?.completions || [];

                              return displayedVotes.map((vote, modelIdx) => {
                                const maxVoteIdx = vote.vote.indexOf(Math.max(...vote.vote));
                                const confidence = Math.max(...vote.vote) * 100;
                                // Use readable model name if available, else shortened cryptic ID
                                const displayName = modelNames[vote.model] || vote.model.slice(0, 8);
                                const isResolved = !!modelNames[vote.model];
                                const isExpanded = expandedVotes.has(modelIdx);
                                // Find matching completion by model ID
                                const completion = completions.find(c => c.model === vote.model);
                                // Handle both streaming (delta) and non-streaming (message) structures
                                const choice = completion?.choices?.[0];
                                const reasoningText = choice?.message?.content || choice?.delta?.content;

                                return (
                                  <div key={modelIdx}>
                                    <div
                                      style={{
                                        display: "flex",
                                        justifyContent: "space-between",
                                        alignItems: isMobile ? "flex-start" : "baseline",
                                        flexDirection: isMobile ? "column" : "row",
                                        gap: isMobile ? "4px" : "0",
                                        marginBottom: "8px",
                                        cursor: reasoningText ? "pointer" : "default",
                                      }}
                                      onClick={() => {
                                        if (!reasoningText) return;
                                        setExpandedVotes(prev => {
                                          const next = new Set(prev);
                                          if (next.has(modelIdx)) {
                                            next.delete(modelIdx);
                                          } else {
                                            next.add(modelIdx);
                                          }
                                          return next;
                                        });
                                      }}
                                    >
                                      <span style={{ fontSize: isMobile ? "12px" : "13px", color: "var(--text)" }}>
                                        {reasoningText && (
                                          <span style={{
                                            display: "inline-block",
                                            width: "16px",
                                            color: "var(--text-muted)",
                                            fontSize: "10px",
                                          }}>
                                            {isExpanded ? "▼" : "▶"}
                                          </span>
                                        )}
                                        <span
                                          className={isResolved ? "model-name" : "model-id"}
                                          style={{
                                            fontFamily: isResolved ? "inherit" : "monospace",
                                            fontSize: isResolved ? (isMobile ? "12px" : "13px") : (isMobile ? "11px" : "12px"),
                                            color: isResolved ? "var(--text)" : "var(--text-muted)",
                                          }}
                                        >
                                          {displayName}
                                        </span>
                                        <span style={{ margin: "0 6px", color: "var(--text-muted)" }}>→</span>
                                        {isMobile ? getOptionLabel(maxVoteIdx).slice(0, 15) + (getOptionLabel(maxVoteIdx).length > 15 ? "…" : "") : getOptionLabel(maxVoteIdx)}
                                      </span>
                                      <span style={{ fontSize: isMobile ? "12px" : "13px" }}>
                                        <span style={{ color: getScoreColor(confidence), fontWeight: 500 }}>
                                          {confidence.toFixed(0)}%
                                        </span>
                                        {!isMobile && (
                                          <span style={{ color: "var(--text-muted)", marginLeft: "8px", fontSize: "11px" }}>
                                            w:{vote.weight}
                                          </span>
                                        )}
                                      </span>
                                    </div>
                                    {/* Progress bar - muted fill, no color */}
                                    <div style={{
                                      height: "6px",
                                      background: "var(--border)",
                                      borderRadius: "3px",
                                      overflow: "hidden",
                                    }}>
                                      <div style={{
                                        height: "100%",
                                        width: `${confidence}%`,
                                        background: "var(--text-muted)",
                                        borderRadius: "3px",
                                        opacity: 0.4,
                                      }} />
                                    </div>
                                    {/* Expanded reasoning */}
                                    {isExpanded && reasoningText && (
                                      <div style={{
                                        marginTop: "8px",
                                        padding: "12px",
                                        background: "var(--page-bg)",
                                        borderRadius: "8px",
                                        fontSize: "12px",
                                        color: "var(--text-muted)",
                                        lineHeight: 1.5,
                                        whiteSpace: "pre-wrap",
                                      }}>
                                        {reasoningText}
                                      </div>
                                    )}
                                  </div>
                                );
                              });
                            })()}
                            {votes.length > 5 && (
                              <button
                                onClick={() => setShowAllModels(!showAllModels)}
                                style={{
                                  background: "none",
                                  border: "none",
                                  padding: 0,
                                  fontSize: "12px",
                                  color: "var(--accent)",
                                  cursor: "pointer",
                                  textAlign: "left",
                                }}
                              >
                                {showAllModels
                                  ? "Show less"
                                  : `+${votes.length - 5} more model${votes.length - 5 !== 1 ? "s" : ""}`
                                }
                              </button>
                            )}
                            </div>
                          </div>

                          {allSimulated && (
                            <p style={{
                              marginTop: "16px",
                              fontSize: "11px",
                              color: "var(--text-muted)",
                              opacity: 0.7,
                            }}>
                              Demo mode — results simulated
                            </p>
                          )}
                        </>
                      );
                    })()}
                  </div>
                )}

                {/* Reasoning Summary */}
                {results.reasoning?.choices?.[0]?.message?.content && (
                  <div style={{
                    padding: isMobile ? "12px" : "16px",
                    background: "var(--page-bg)",
                    borderRadius: "6px",
                    border: "1px solid var(--border)",
                  }}>
                    <p style={{
                      fontSize: isMobile ? "12px" : "13px",
                      color: "var(--text-muted)",
                      marginBottom: isMobile ? "8px" : "12px",
                    }}>
                      Reasoning Summary
                    </p>
                    <p style={{
                      fontSize: isMobile ? "13px" : "14px",
                      color: "var(--text)",
                      lineHeight: 1.6,
                      whiteSpace: "pre-wrap",
                    }}>
                      {results.reasoning.choices[0].message.content}
                    </p>
                  </div>
                )}

                {/* Usage & Cost */}
                {results.usage && (
                  <div style={{
                    padding: isMobile ? "10px 12px" : "12px 16px",
                    background: "var(--page-bg)",
                    borderRadius: "6px",
                    fontSize: isMobile ? "12px" : "13px",
                    color: "var(--text-muted)",
                    display: "flex",
                    flexWrap: "wrap",
                    gap: isMobile ? "12px" : "16px",
                  }}>
                    <span>
                      {results.usage.total_tokens.toLocaleString()} tokens
                    </span>
                    {results.usage.cost !== undefined && (
                      <span style={{ color: "var(--text)" }}>
                        ${results.usage.cost.toFixed(4)}
                      </span>
                    )}
                    {!isMobile && results.usage.total_cost !== undefined && results.usage.total_cost !== results.usage.cost && (
                      <span>
                        (${results.usage.total_cost.toFixed(4)} total)
                      </span>
                    )}
                  </div>
                )}

              </div>
            )}
          </div>
        )}

    </div>
  );
}
