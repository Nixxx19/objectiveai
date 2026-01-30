"use client";

import { useState, useEffect, use } from "react";
import Link from "next/link";
import { getClient, Functions, deriveDisplayName, DEV_EXECUTION_OPTIONS } from "../../../lib/objectiveai";

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

export default function FunctionDetailPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = use(params);

  // Parse slug (format: owner--repository)
  const [owner, repository] = slug.includes("--")
    ? slug.split("--")
    : ["unknown", slug];

  const [functionDetails, setFunctionDetails] = useState<FunctionDetails | null>(null);
  const [profileInfo, setProfileInfo] = useState<{ owner: string; repository: string; commit: string } | null>(null);
  const [isLoadingDetails, setIsLoadingDetails] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  const [formData, setFormData] = useState<Record<string, unknown>>({});
  const [rawInput, setRawInput] = useState("{}");
  const [isRunning, setIsRunning] = useState(false);
  const [isMobile, setIsMobile] = useState(false);
  const [isSaved, setIsSaved] = useState(false);
  const [showPinnedColor, setShowPinnedColor] = useState(false);
  const [results, setResults] = useState<{
    output?: number | number[];
    usage?: { prompt_tokens: number; completion_tokens: number; total_tokens: number };
    error?: string;
  } | null>(null);
  const [runError, setRunError] = useState<string | null>(null);

  // Fetch function details
  useEffect(() => {
    async function fetchDetails() {
      try {
        setIsLoadingDetails(true);
        setLoadError(null);
        const client = getClient();

        // First get the function-profile pair
        const pairs = await Functions.listPairs(client);
        const pair = pairs.data.find(
          p => p.function.owner === owner && p.function.repository === repository
        );

        if (!pair) {
          throw new Error(`Function ${owner}/${repository} not found`);
        }

        // Store profile info for execution
        setProfileInfo(pair.profile);

        // Fetch full function details
        const details = await Functions.retrieve(
          client,
          pair.function.owner,
          pair.function.repository,
          pair.function.commit
        );

        const category = details.type === "vector.function" ? "Ranking" : "Scoring";

        setFunctionDetails({
          owner: pair.function.owner,
          repository: pair.function.repository,
          commit: pair.function.commit,
          name: deriveDisplayName(pair.function.repository),
          description: details.description || `${deriveDisplayName(pair.function.repository)} function`,
          category,
          type: details.type as "scalar.function" | "vector.function",
          inputSchema: (details as { input_schema?: Record<string, unknown> }).input_schema || null,
        });
      } catch (err) {
        console.error("Failed to fetch function details:", err);
        setLoadError(err instanceof Error ? err.message : "Failed to load function");
      } finally {
        setIsLoadingDetails(false);
      }
    }

    fetchDetails();
  }, [owner, repository]);

  // Load saved state from localStorage
  useEffect(() => {
    const savedLibrary = localStorage.getItem("pinned-functions");
    if (savedLibrary) {
      const library = JSON.parse(savedLibrary);
      setIsSaved(library.includes(slug));
    }
  }, [slug]);

  // Toggle save state
  const toggleSave = () => {
    const savedLibrary = localStorage.getItem("pinned-functions");
    const library = savedLibrary ? JSON.parse(savedLibrary) : [];

    if (isSaved) {
      const updated = library.filter((s: string) => s !== slug);
      localStorage.setItem("pinned-functions", JSON.stringify(updated));
      setIsSaved(false);
    } else {
      library.push(slug);
      localStorage.setItem("pinned-functions", JSON.stringify(library));
      setIsSaved(true);
      setShowPinnedColor(true);
      setTimeout(() => setShowPinnedColor(false), 1000);
    }
  };

  // Track viewport size
  useEffect(() => {
    const checkMobile = () => {
      setIsMobile(window.innerWidth <= 768);
    };
    checkMobile();
    window.addEventListener("resize", checkMobile);
    return () => window.removeEventListener("resize", checkMobile);
  }, []);

  // Execute function via SDK
  const handleRun = async () => {
    if (!functionDetails || !profileInfo) return;

    setIsRunning(true);
    setRunError(null);
    setResults(null);

    try {
      const client = getClient();

      const result = await Functions.Executions.create(
        client,
        {
          owner: functionDetails.owner,
          repository: functionDetails.repository,
          commit: functionDetails.commit,
        },
        {
          owner: profileInfo.owner,
          repository: profileInfo.repository,
          commit: profileInfo.commit,
        },
        {
          input: formData,
          ...DEV_EXECUTION_OPTIONS,
        }
      );

      // Handle the response
      if ("output" in result) {
        setResults({
          output: result.output as number | number[],
          usage: result.usage,
        });
      }
    } catch (err) {
      console.error("Function execution failed:", err);
      setRunError(err instanceof Error ? err.message : "Execution failed");
    } finally {
      setIsRunning(false);
    }
  };

  // Build input fields from schema
  const renderInputFields = () => {
    if (!functionDetails?.inputSchema) {
      // Fallback: single text input
      return (
        <div>
          <label style={{
            display: "block",
            fontSize: "14px",
            fontWeight: 600,
            marginBottom: "8px",
            color: "var(--text)",
          }}>
            Input
            <span style={{
              fontWeight: 400,
              color: "var(--text-muted)",
              marginLeft: "8px",
            }}>
              Your input data (JSON)
            </span>
          </label>
          <div className="aiTextField">
            <textarea
              placeholder="Enter input as JSON..."
              value={rawInput}
              onChange={(e) => {
                setRawInput(e.target.value);
                try {
                  setFormData(JSON.parse(e.target.value));
                } catch {
                  // Keep typing, will parse on submit
                }
              }}
              rows={6}
            />
          </div>
        </div>
      );
    }

    // Parse input_schema to generate fields
    const schema = functionDetails.inputSchema as {
      type?: string;
      properties?: Record<string, { type?: string; description?: string }>;
    };

    if (schema.type === "object" && schema.properties) {
      return Object.entries(schema.properties).map(([key, prop]) => (
        <div key={key}>
          <label style={{
            display: "block",
            fontSize: "14px",
            fontWeight: 600,
            marginBottom: "8px",
            color: "var(--text)",
          }}>
            {key.charAt(0).toUpperCase() + key.slice(1)}
            {prop.description && (
              <span style={{
                fontWeight: 400,
                color: "var(--text-muted)",
                marginLeft: "8px",
                display: isMobile ? "block" : "inline",
                marginTop: isMobile ? "4px" : "0",
              }}>
                {prop.description}
              </span>
            )}
          </label>

          {prop.type === "string" && (
            <div className="aiTextField">
              <textarea
                placeholder={`Enter ${key}...`}
                value={(formData[key] as string) || ""}
                onChange={(e) => setFormData({ ...formData, [key]: e.target.value })}
                rows={3}
              />
            </div>
          )}

          {(prop.type === "number" || prop.type === "integer") && (
            <div className="aiTextField">
              <input
                type="number"
                placeholder={`Enter ${key}...`}
                value={(formData[key] as number) || ""}
                onChange={(e) => setFormData({ ...formData, [key]: parseFloat(e.target.value) })}
              />
            </div>
          )}

          {prop.type === "array" && (
            <div className="aiTextField">
              <textarea
                placeholder={`Enter ${key} (comma-separated or JSON array)...`}
                value={(formData[key + "_raw"] as string) ?? (Array.isArray(formData[key]) ? formData[key].join(", ") : "")}
                onChange={(e) => {
                  const val = e.target.value;
                  // Store raw value for display
                  setFormData(prev => ({ ...prev, [key + "_raw"]: val }));
                  // Try to parse as JSON array, otherwise split by comma
                  try {
                    const parsed = JSON.parse(val);
                    if (Array.isArray(parsed)) {
                      setFormData(prev => ({ ...prev, [key]: parsed }));
                    }
                  } catch {
                    // Split by comma for simple input
                    const items = val.split(",").map(s => s.trim()).filter(Boolean);
                    setFormData(prev => ({ ...prev, [key]: items }));
                  }
                }}
                rows={3}
              />
            </div>
          )}

          {/* Fallback for unknown types */}
          {prop.type !== "string" && prop.type !== "number" && prop.type !== "integer" && prop.type !== "array" && (
            <div className="aiTextField">
              <textarea
                placeholder={`Enter ${key}...`}
                value={(formData[key] as string) || ""}
                onChange={(e) => setFormData({ ...formData, [key]: e.target.value })}
                rows={3}
              />
            </div>
          )}
        </div>
      ));
    }

    // Fallback for other schema types
    return (
      <div>
        <label style={{
          display: "block",
          fontSize: "14px",
          fontWeight: 600,
          marginBottom: "8px",
          color: "var(--text)",
        }}>
          Input
        </label>
        <div className="aiTextField">
          <textarea
            placeholder="Enter input as JSON..."
            value={rawInput}
            onChange={(e) => {
              setRawInput(e.target.value);
              try {
                setFormData(JSON.parse(e.target.value));
              } catch {
                // Invalid JSON, keep typing
              }
            }}
            rows={6}
          />
        </div>
      </div>
    );
  };

  // Render results based on output type
  const renderResults = () => {
    if (!results?.output) return null;

    const output = results.output;

    // Scalar output (single number)
    if (typeof output === "number") {
      return (
        <div>
          <p style={{
            fontSize: "13px",
            color: "var(--text-muted)",
            marginBottom: "8px",
          }}>
            Score
          </p>
          <p style={{
            fontSize: isMobile ? "36px" : "48px",
            fontWeight: 700,
            color: "var(--accent)",
            lineHeight: 1,
          }}>
            {(output * 100).toFixed(1)}%
          </p>
        </div>
      );
    }

    // Vector output (array of numbers)
    if (Array.isArray(output)) {
      const getScoreColor = (score: number) => {
        const pct = score * 100;
        if (pct >= 35) return "rgb(34, 197, 94)";
        if (pct >= 20) return "rgb(234, 179, 8)";
        if (pct >= 10) return "rgb(249, 115, 22)";
        return "rgb(239, 68, 68)";
      };

      return (
        <div>
          <p style={{
            fontSize: "13px",
            color: "var(--text-muted)",
            marginBottom: "12px",
          }}>
            Rankings
          </p>
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            {output
              .map((score, i) => ({ index: i, score }))
              .sort((a, b) => b.score - a.score)
              .map((item, rank) => (
                <div key={item.index} style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "12px",
                  padding: "12px 16px",
                  background: "var(--page-bg)",
                  borderRadius: "12px",
                }}>
                  <span style={{
                    fontSize: "14px",
                    fontWeight: 700,
                    color: getScoreColor(item.score),
                    width: "40px",
                    flexShrink: 0,
                  }}>
                    {(item.score * 100).toFixed(0)}%
                  </span>
                  <span style={{ flex: 1, fontSize: "14px" }}>
                    Option {item.index + 1}
                  </span>
                  <span style={{
                    fontSize: "12px",
                    color: "var(--text-muted)",
                  }}>
                    #{rank + 1}
                  </span>
                </div>
              ))}
          </div>
        </div>
      );
    }

    return null;
  };

  const spinnerStyle = `
    @keyframes spin {
      to { transform: rotate(360deg); }
    }
  `;

  // Loading state
  if (isLoadingDetails) {
    return (
      <div className="page">
        <style dangerouslySetInnerHTML={{ __html: spinnerStyle }} />
        <div style={{
          maxWidth: "1400px",
          margin: "0 auto",
          padding: "0 32px",
          textAlign: "center",
          paddingTop: "100px",
        }}>
          <div style={{
            width: "40px",
            height: "40px",
            border: "3px solid var(--border)",
            borderTopColor: "var(--accent)",
            borderRadius: "50%",
            margin: "0 auto 16px",
            animation: "spin 1s linear infinite",
          }} />
          <p style={{ color: "var(--text-muted)" }}>Loading function...</p>
        </div>
      </div>
    );
  }

  // Error state
  if (loadError || !functionDetails) {
    return (
      <div className="page">
        <div style={{
          maxWidth: "1400px",
          margin: "0 auto",
          padding: "0 32px",
          textAlign: "center",
          paddingTop: "100px",
        }}>
          <p style={{ color: "#ef4444", marginBottom: "8px" }}>Failed to load function</p>
          <p style={{ color: "var(--text-muted)", marginBottom: "24px" }}>{loadError}</p>
          <Link href="/functions" style={{ color: "var(--accent)" }}>
            Back to Functions
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="page">
      <style dangerouslySetInnerHTML={{ __html: spinnerStyle }} />

      <div style={{
        maxWidth: "1400px",
        margin: "0 auto",
        padding: isMobile ? "0 20px" : "0 32px",
      }}>
        {/* Breadcrumb Row with Pin */}
        <div style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "baseline",
          marginBottom: "20px",
          fontSize: "14px",
        }}>
          <nav style={{
            display: "flex",
            gap: "8px",
            color: "var(--text-muted)",
            flexWrap: "wrap",
          }}>
            <Link href="/functions" style={{ color: "var(--accent)", textDecoration: "none" }}>
              Functions
            </Link>
            <span>/</span>
            <span>{functionDetails.name}</span>
          </nav>
          <button
            onClick={toggleSave}
            style={{
              background: "none",
              border: "none",
              padding: 0,
              cursor: "pointer",
              fontSize: "inherit",
              color: showPinnedColor ? "var(--accent)" : "var(--text-muted)",
              opacity: 0.7,
              transition: showPinnedColor ? "color 0.15s ease-in" : "color 0.5s ease-out",
            }}
          >
            {isSaved ? "Pinned Function" : "Pin Function"}
          </button>
        </div>

        {/* Header */}
        <div style={{ marginBottom: isMobile ? "20px" : "28px" }}>
          <h1 className="heading2" style={{ marginBottom: "4px" }}>
            {functionDetails.name}
          </h1>
          <p style={{
            fontSize: isMobile ? "15px" : "17px",
            color: "var(--text-muted)",
            maxWidth: "700px",
            lineHeight: 1.6,
            marginBottom: "8px",
          }}>
            {functionDetails.description}
          </p>
          <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
            <span className="tag" style={{ display: "inline-block" }}>{functionDetails.category}</span>
            <span style={{
              fontSize: "12px",
              padding: "4px 12px",
              background: "var(--border)",
              borderRadius: "12px",
              color: "var(--text-muted)",
            }}>
              {functionDetails.owner}/{functionDetails.repository}
            </span>
          </div>
        </div>

        {/* Main Layout */}
        <div style={{
          display: isMobile ? "flex" : "grid",
          flexDirection: "column",
          gridTemplateColumns: "1fr 1fr",
          gap: isMobile ? "24px" : "32px",
          alignItems: "start",
        }}>
          {/* Left - Input */}
          <div className="card">
            <h3 style={{
              fontSize: "12px",
              fontWeight: 600,
              marginBottom: "24px",
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "var(--text-muted)",
            }}>
              Input
            </h3>

            <div style={{ display: "flex", flexDirection: "column", gap: "24px" }}>
              {renderInputFields()}
            </div>

            <button
              className="pillBtn"
              onClick={handleRun}
              disabled={isRunning}
              style={{
                width: "100%",
                marginTop: "32px",
                opacity: isRunning ? 0.7 : 1,
              }}
            >
              {isRunning ? "Running..." : "Execute"}
            </button>
          </div>

          {/* Right - Results */}
          <div className="card">
            <h3 style={{
              fontSize: "12px",
              fontWeight: 600,
              marginBottom: "24px",
              textTransform: "uppercase",
              letterSpacing: "0.05em",
              color: "var(--text-muted)",
            }}>
              Output
            </h3>

            {!results && !isRunning && !runError && (
              <div style={{
                textAlign: "center",
                padding: isMobile ? "40px 20px" : "60px 20px",
                color: "var(--text-muted)",
              }}>
                <p style={{ marginBottom: "8px", fontSize: "24px" }}>—</p>
                <p style={{ fontSize: "14px" }}>Run the function to see results</p>
              </div>
            )}

            {isRunning && (
              <div style={{
                textAlign: "center",
                padding: isMobile ? "40px 20px" : "60px 20px",
                color: "var(--text-muted)",
              }}>
                <div style={{
                  width: "40px",
                  height: "40px",
                  border: "3px solid var(--border)",
                  borderTopColor: "var(--accent)",
                  borderRadius: "50%",
                  margin: "0 auto 16px",
                  animation: "spin 1s linear infinite",
                }} />
                <p style={{ fontSize: "14px" }}>Processing...</p>
              </div>
            )}

            {runError && !isRunning && (
              <div style={{
                textAlign: "center",
                padding: isMobile ? "40px 20px" : "60px 20px",
              }}>
                <p style={{ color: "#ef4444", marginBottom: "8px" }}>
                  {runError.includes("401") ? "Not authenticated" : "Execution failed"}
                </p>
                <p style={{ fontSize: "14px", color: "var(--text-muted)" }}>
                  {runError.includes("401")
                    ? "API key missing or invalid. Contact admin to set up access."
                    : runError}
                </p>
              </div>
            )}

            {results && !isRunning && (
              <div style={{ display: "flex", flexDirection: "column", gap: "32px" }}>
                {renderResults()}

                {/* Usage info */}
                {results.usage && (
                  <div style={{
                    padding: "12px 16px",
                    background: "var(--page-bg)",
                    borderRadius: "12px",
                    fontSize: "13px",
                    color: "var(--text-muted)",
                  }}>
                    <p>Tokens: {results.usage.total_tokens} ({results.usage.prompt_tokens} prompt + {results.usage.completion_tokens} completion)</p>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
