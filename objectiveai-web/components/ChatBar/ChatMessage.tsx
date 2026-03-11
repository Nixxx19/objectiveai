"use client";

import type { ChatMessage as ChatMessageType } from "./types";

interface ChatMessageProps {
  message: ChatMessageType;
  isMobile: boolean;
}

/** Score color — same thresholds as function tree. */
function scoreColor(pct: number): string {
  if (pct >= 66) return "var(--color-success)";
  if (pct >= 33) return "var(--color-warning)";
  if (pct >= 15) return "var(--color-danger)";
  return "var(--color-error)";
}

export function ChatMessageBubble({ message, isMobile }: ChatMessageProps) {
  const isUser = message.role === "user";

  // Tool execution indicator
  if (message.toolCall && !message.content) {
    const result = message.executionResult;
    return (
      <div style={{
        display: "flex",
        flexDirection: "column",
        gap: "4px",
        padding: "8px 12px",
        fontSize: "12px",
        color: "var(--text-muted)",
        borderLeft: "2px solid var(--border)",
        marginLeft: "4px",
      }}>
        <span style={{ fontFamily: "monospace", fontSize: "11px" }}>
          execute_function({JSON.stringify(message.toolCall.input).slice(0, 60)}
          {JSON.stringify(message.toolCall.input).length > 60 ? "..." : ""})
        </span>
        {result ? (
          result.error ? (
            <span style={{ color: "var(--color-error)" }}>Error: {result.error}</span>
          ) : result.output != null ? (
            <span style={{ color: "var(--color-success)" }}>
              {typeof result.output === "number"
                ? `Score: ${(result.output * 100).toFixed(1)}%`
                : `Ranking: [${(result.output as number[]).map(s => (s * 100).toFixed(0) + "%").join(", ")}]`}
            </span>
          ) : null
        ) : (
          <span style={{ display: "flex", alignItems: "center", gap: "6px" }}>
            <span style={{
              width: "10px",
              height: "10px",
              border: "1.5px solid var(--border)",
              borderTopColor: "var(--accent)",
              borderRadius: "50%",
              animation: "spin 1s linear infinite",
            }} />
            Executing...
          </span>
        )}
      </div>
    );
  }

  if (!message.content) return null;

  return (
    <div style={{
      display: "flex",
      justifyContent: isUser ? "flex-end" : "flex-start",
    }}>
      <div style={{
        maxWidth: isMobile ? "85%" : "80%",
        padding: "8px 12px",
        borderRadius: "12px",
        fontSize: "13px",
        lineHeight: 1.5,
        ...(isUser
          ? {
              background: "var(--accent)",
              color: "#fff",
              borderBottomRightRadius: "4px",
            }
          : {
              background: "var(--card-bg)",
              color: "var(--text)",
              border: "1px solid var(--border)",
              borderBottomLeftRadius: "4px",
            }),
      }}>
        {message.content}
      </div>
    </div>
  );
}
