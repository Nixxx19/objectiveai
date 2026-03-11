"use client";

import { useState, useRef, useEffect, useCallback } from "react";
import { ChatMessageBubble } from "./ChatMessage";
import type { ChatMessage, ChatState } from "./types";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface ProfileOption {
  owner: string;
  repository: string;
  label: string;
}

interface ChatBarProps {
  messages: ChatMessage[];
  chatState: ChatState;
  onSend: (text: string) => void;
  onClear: () => void;
  profiles: ProfileOption[];
  selectedProfileIndex: number;
  onProfileChange: (index: number) => void;
  demoMode: boolean;
  onDemoModeChange: (enabled: boolean) => void;
  reasoningEnabled: boolean;
  onReasoningChange: (enabled: boolean) => void;
  isMobile: boolean;
  isExecuting: boolean;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export function ChatBar({
  messages,
  chatState,
  onSend,
  onClear,
  profiles,
  selectedProfileIndex,
  onProfileChange,
  demoMode,
  onDemoModeChange,
  reasoningEnabled,
  onReasoningChange,
  isMobile,
  isExecuting,
}: ChatBarProps) {
  const [input, setInput] = useState("");
  const [expanded, setExpanded] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const isBusy = chatState === "thinking" || chatState === "executing" || isExecuting;

  // Auto-scroll messages
  useEffect(() => {
    if (expanded && messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages, expanded]);

  // Auto-resize textarea
  const handleInputChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    const el = e.target;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 140) + "px";
  }, []);

  const handleSend = useCallback(() => {
    if (!input.trim() || isBusy) return;
    onSend(input);
    setInput("");
    if (inputRef.current) {
      inputRef.current.style.height = "auto";
    }
    if (messages.length === 0) setExpanded(true);
  }, [input, isBusy, onSend, messages.length]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }, [handleSend]);

  return (
    <div style={{
      width: isMobile ? "calc(100% - 24px)" : "min(640px, calc(100% - 48px))",
      margin: "0 auto",
    }}>
      {/* Single cohesive card */}
      <div style={{
        background: "var(--card-bg)",
        borderRadius: "16px",
        border: "1px solid var(--border)",
        boxShadow: "0 4px 24px rgba(0,0,0,0.12), 0 1px 4px rgba(0,0,0,0.08)",
        backdropFilter: "blur(20px)",
        overflow: "hidden",
      }}>
        {/* Expandable conversation area — inside the card */}
        {expanded && messages.length > 0 && (
          <div style={{
            maxHeight: "35vh",
            overflowY: "auto",
            padding: isMobile ? "14px 16px" : "16px 20px",
            borderBottom: "1px solid var(--border)",
            display: "flex",
            flexDirection: "column",
            gap: "10px",
          }}>
            {messages.map(msg => (
              <ChatMessageBubble key={msg.id} message={msg} isMobile={isMobile} />
            ))}
            <div ref={messagesEndRef} />
          </div>
        )}

        {/* Control strip */}
        <div style={{
          display: "flex",
          alignItems: "center",
          gap: isMobile ? "8px" : "12px",
          padding: isMobile ? "8px 16px" : "8px 20px",
          fontSize: "12px",
          borderBottom: "1px solid var(--border)",
        }}>
          {/* Expand/collapse */}
          {messages.length > 0 && (
            <button
              onClick={() => setExpanded(v => !v)}
              style={{
                background: "none",
                border: "none",
                padding: "4px 6px",
                cursor: "pointer",
                color: "var(--text-muted)",
                fontSize: "11px",
                display: "flex",
                alignItems: "center",
                gap: "5px",
              }}
              title={expanded ? "Collapse" : "Expand"}
            >
              <span style={{
                transform: expanded ? "rotate(180deg)" : "rotate(0deg)",
                transition: "transform 0.15s ease",
                display: "inline-block",
              }}>
                ▲
              </span>
              {!expanded && (
                <span style={{
                  background: "var(--accent)",
                  color: "#fff",
                  borderRadius: "8px",
                  padding: "1px 6px",
                  fontSize: "10px",
                  fontWeight: 600,
                  lineHeight: "16px",
                }}>
                  {messages.length}
                </span>
              )}
            </button>
          )}

          {/* Profile */}
          <select
            value={selectedProfileIndex}
            onChange={(e) => onProfileChange(parseInt(e.target.value, 10))}
            style={{
              background: "transparent",
              border: "none",
              fontSize: "12px",
              color: "var(--text-muted)",
              cursor: "pointer",
              padding: "2px 4px",
              maxWidth: isMobile ? "100px" : "140px",
            }}
          >
            {profiles.map((p, idx) => (
              <option key={`${p.owner}/${p.repository}`} value={idx}>
                {p.label}
              </option>
            ))}
          </select>

          <div style={{ flex: 1 }} />

          {/* Demo */}
          <label style={{
            display: "flex",
            alignItems: "center",
            gap: "5px",
            cursor: "pointer",
            color: "var(--text-muted)",
            fontSize: "12px",
          }}>
            <input
              type="checkbox"
              checked={demoMode}
              onChange={(e) => onDemoModeChange(e.target.checked)}
              style={{ width: "13px", height: "13px", accentColor: "var(--accent)" }}
            />
            Demo
          </label>

          {/* Reasoning */}
          {!isMobile && (
            <label style={{
              display: "flex",
              alignItems: "center",
              gap: "5px",
              cursor: "pointer",
              color: "var(--text-muted)",
              fontSize: "12px",
            }}>
              <input
                type="checkbox"
                checked={reasoningEnabled}
                onChange={(e) => onReasoningChange(e.target.checked)}
                style={{ width: "13px", height: "13px", accentColor: "var(--accent)" }}
              />
              Reasoning
            </label>
          )}

          {/* Clear */}
          {messages.length > 0 && (
            <button
              onClick={onClear}
              style={{
                background: "none",
                border: "none",
                padding: "4px 8px",
                cursor: "pointer",
                fontSize: "12px",
                color: "var(--text-muted)",
              }}
            >
              Clear
            </button>
          )}
        </div>

        {/* Input area — the prominent inner field */}
        <div style={{
          display: "flex",
          alignItems: "flex-end",
          gap: "10px",
          padding: isMobile ? "12px 12px 14px" : "14px 16px 16px",
        }}>
          <div style={{
            flex: 1,
            background: "var(--page-bg)",
            borderRadius: "12px",
            border: "1px solid var(--border)",
            padding: "10px 14px",
            minHeight: "48px",
            display: "flex",
            alignItems: "center",
          }}>
            <textarea
              ref={inputRef}
              value={input}
              onChange={handleInputChange}
              onKeyDown={handleKeyDown}
              placeholder={isBusy ? "Waiting..." : "Describe what you want to score or rank..."}
              disabled={isBusy}
              rows={1}
              style={{
                width: "100%",
                resize: "none",
                border: "none",
                outline: "none",
                background: "transparent",
                fontSize: "14px",
                lineHeight: 1.5,
                color: "var(--text)",
                padding: 0,
                fontFamily: "inherit",
                maxHeight: "140px",
              }}
            />
          </div>
          <button
            onClick={handleSend}
            disabled={isBusy || !input.trim()}
            style={{
              background: input.trim() && !isBusy ? "var(--accent)" : "var(--border)",
              border: "none",
              borderRadius: "12px",
              width: "44px",
              height: "44px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: input.trim() && !isBusy ? "pointer" : "default",
              flexShrink: 0,
              transition: "background 0.15s ease",
            }}
            title="Send"
          >
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke={input.trim() && !isBusy ? "#fff" : "var(--text-muted)"}
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <line x1="22" y1="2" x2="11" y2="13" />
              <polygon points="22 2 15 22 11 13 2 9 22 2" />
            </svg>
          </button>
        </div>

        {/* Progress bar when busy */}
        {isBusy && (
          <div style={{
            height: "2px",
            background: "var(--border)",
            overflow: "hidden",
          }}>
            <div style={{
              height: "100%",
              width: "30%",
              background: "var(--accent)",
              borderRadius: "1px",
              animation: "chatbar-progress 1.5s ease-in-out infinite",
            }} />
          </div>
        )}
      </div>

      <style>{`
        @keyframes chatbar-progress {
          0% { transform: translateX(-100%); }
          50% { transform: translateX(233%); }
          100% { transform: translateX(-100%); }
        }
      `}</style>
    </div>
  );
}
