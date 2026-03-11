"use client";

import { useState, useCallback, useRef } from "react";
import { useObjectiveAI } from "../../hooks/useObjectiveAI";
import { Chat, Functions } from "objectiveai";
import { DEV_EXECUTION_OPTIONS } from "../../lib/objectiveai";
import type { ChatMessage, ChatState, ToolCallInfo, ExecutionResult } from "./types";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface FunctionMeta {
  owner: string;
  repository: string;
  commit: string;
  name: string;
  description: string;
  type: "scalar.function" | "vector.function";
  inputSchema: Record<string, unknown> | null;
}

interface ProfileRef {
  owner: string;
  repository: string;
  commit: string | null;
}

interface OrchestrationOptions {
  functionMeta: FunctionMeta | null;
  profile: ProfileRef;
  demoMode: boolean;
  reasoningEnabled: boolean;
  reasoningModel: string;
  /** Called with streaming execution data (same shape as page's setResults). */
  onExecutionUpdate: (data: unknown) => void;
  /** Called when execution starts (to set isRunning on page). */
  onExecutionStart: () => void;
  /** Called when execution ends. */
  onExecutionEnd: () => void;
}

let _msgId = 0;
function msgId(): string {
  return `msg-${++_msgId}-${Date.now()}`;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export function useChatOrchestration(options: OrchestrationOptions) {
  const {
    functionMeta,
    profile,
    demoMode,
    reasoningEnabled,
    reasoningModel,
    onExecutionUpdate,
    onExecutionStart,
    onExecutionEnd,
  } = options;

  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [chatState, setChatState] = useState<ChatState>("idle");
  const { getClient } = useObjectiveAI();
  const abortRef = useRef<AbortController | null>(null);

  // Build system prompt from function metadata
  const buildSystemPrompt = useCallback((): string => {
    if (!functionMeta) return "You are a helpful assistant.";

    const schemaBlock = functionMeta.inputSchema
      ? `\n\nThe function accepts the following input schema:\n\`\`\`json\n${JSON.stringify(functionMeta.inputSchema, null, 2)}\n\`\`\``
      : "\n\nThe function accepts freeform JSON input.";

    const typeDesc = functionMeta.type === "scalar.function"
      ? "It returns a scalar score between 0 and 1."
      : "It returns a vector of scores that sum to approximately 1 (a ranking).";

    return `You are helping a user interact with an ObjectiveAI function called "${functionMeta.name}" (${functionMeta.owner}/${functionMeta.repository}).

${functionMeta.description}

${typeDesc}${schemaBlock}

Use the execute_function tool when you have enough information to run the function. If the user's request is unclear or missing required fields, ask a clarifying question. Keep responses concise.`;
  }, [functionMeta]);

  // Build tool definitions from input schema
  const buildTools = useCallback(() => {
    if (!functionMeta) return [];

    const parameters = functionMeta.inputSchema
      ? { ...functionMeta.inputSchema }
      : { type: "object" as const, properties: {} };

    return [
      {
        type: "function" as const,
        function: {
          name: "execute_function",
          description: `Execute the ${functionMeta.name} function with structured input.`,
          parameters,
        },
      },
    ];
  }, [functionMeta]);

  // Execute the function (called when LLM returns a tool call)
  const executeFunction = useCallback(async (
    input: Record<string, unknown>,
  ): Promise<ExecutionResult> => {
    if (!functionMeta) return { output: null, error: "No function loaded" };

    onExecutionStart();

    try {
      const client = await getClient();

      const executionBody = {
        input: input as Parameters<typeof Functions.Executions.create>[3]["input"],
        stream: true as const,
        from_cache: DEV_EXECUTION_OPTIONS.from_cache,
        from_rng: demoMode,
        reasoning: reasoningEnabled
          ? { model: { model: reasoningModel, output_mode: "instruction" as const } }
          : undefined,
      };

      const stream = await Functions.Executions.create(
        client,
        {
          remote: "github",
          owner: functionMeta.owner,
          repository: functionMeta.repository,
          commit: functionMeta.commit,
        },
        {
          remote: "github",
          owner: profile.owner,
          repository: profile.repository,
          commit: profile.commit,
        },
        executionBody,
      );

      let finalOutput: number | number[] | null = null;

      for await (const chunk of stream) {
        if (chunk.error) {
          throw new Error(
            typeof chunk.error === "object" ? JSON.stringify(chunk.error) : String(chunk.error),
          );
        }
        if (chunk.output !== undefined) {
          finalOutput = chunk.output as number | number[];
        }
        // Forward streaming data to the page for tree updates
        onExecutionUpdate(chunk);
      }

      return { output: finalOutput };
    } catch (err) {
      const msg = err instanceof Error ? err.message : "Execution failed";
      return { output: null, error: msg };
    } finally {
      onExecutionEnd();
    }
  }, [functionMeta, profile, demoMode, reasoningEnabled, reasoningModel, getClient, onExecutionUpdate, onExecutionStart, onExecutionEnd]);

  // Send a user message and handle the LLM response
  const sendMessage = useCallback(async (text: string) => {
    if (!text.trim() || !functionMeta) return;

    // Add user message
    const userMsg: ChatMessage = {
      id: msgId(),
      role: "user",
      content: text.trim(),
      ts: Date.now(),
    };
    setMessages(prev => [...prev, userMsg]);
    setChatState("thinking");

    try {
      const client = await getClient();

      // Build conversation for the API (all messages so far + new user message)
      const apiMessages: Array<{ role: string; content: string }> = [
        { role: "system", content: buildSystemPrompt() },
      ];
      // Include previous messages (skip tool-call-only messages)
      for (const m of [...messages, userMsg]) {
        if (m.role === "system") continue;
        if (m.content) {
          apiMessages.push({ role: m.role, content: m.content });
        }
      }

      const tools = buildTools();

      // Call Chat.Completions (non-streaming for simplicity in v1)
      const response = await Chat.Completions.create(client, {
        model: { model: "openai/gpt-4o-mini", output_mode: "instruction" as const },
        messages: apiMessages as Parameters<typeof Chat.Completions.create>[1]["messages"],
        tools: tools as Parameters<typeof Chat.Completions.create>[1]["tools"],
        tool_choice: "auto",
        upstreams: ["open_router"],
      });

      const choice = response.choices?.[0];
      if (!choice) {
        throw new Error("No response from model");
      }

      const toolCalls = choice.message?.tool_calls;

      if (toolCalls && toolCalls.length > 0) {
        // LLM wants to execute the function
        const tc = toolCalls[0];
        const args = tc.function?.arguments;
        const input = args ? JSON.parse(args) : {};

        const toolCallInfo: ToolCallInfo = {
          input,
          profile: `${profile.owner}/${profile.repository}`,
        };

        // Add tool call message (shows as execution indicator)
        const toolMsg: ChatMessage = {
          id: msgId(),
          role: "assistant",
          content: null,
          toolCall: toolCallInfo,
          ts: Date.now(),
        };
        setMessages(prev => [...prev, toolMsg]);
        setChatState("executing");

        // Execute the function
        const result = await executeFunction(input);

        // Update tool message with result
        setMessages(prev =>
          prev.map(m =>
            m.id === toolMsg.id ? { ...m, executionResult: result } : m,
          ),
        );

        // Generate a summary response from the LLM
        const summaryMessages = [
          ...apiMessages,
          {
            role: "assistant",
            content: null,
            tool_calls: [{ id: tc.id, type: "function", function: { name: "execute_function", arguments: args || "{}" } }],
          },
          {
            role: "tool",
            content: JSON.stringify(result),
            tool_call_id: tc.id,
          },
        ];

        try {
          const summaryResponse = await Chat.Completions.create(client, {
            model: { model: "openai/gpt-4o-mini", output_mode: "instruction" as const },
            messages: summaryMessages as Parameters<typeof Chat.Completions.create>[1]["messages"],
            tools: tools as Parameters<typeof Chat.Completions.create>[1]["tools"],
            upstreams: ["open_router"],
          });

          const summaryContent = summaryResponse.choices?.[0]?.message?.content;
          if (summaryContent) {
            setMessages(prev => [...prev, {
              id: msgId(),
              role: "assistant",
              content: summaryContent,
              ts: Date.now(),
            }]);
          }
        } catch {
          // Summary is best-effort — don't fail the whole flow
        }

        setChatState("complete");
      } else {
        // LLM responded with text (clarification or conversation)
        const content = choice.message?.content || "I'm not sure how to help with that.";
        setMessages(prev => [...prev, {
          id: msgId(),
          role: "assistant",
          content,
          ts: Date.now(),
        }]);
        setChatState("idle");
      }
    } catch (err) {
      const errMsg = err instanceof Error ? err.message : "Something went wrong";
      setMessages(prev => [...prev, {
        id: msgId(),
        role: "assistant",
        content: `Error: ${errMsg}`,
        ts: Date.now(),
      }]);
      setChatState("error");
    }
  }, [functionMeta, messages, getClient, buildSystemPrompt, buildTools, executeFunction, profile]);

  const clearMessages = useCallback(() => {
    setMessages([]);
    setChatState("idle");
  }, []);

  return {
    messages,
    chatState,
    sendMessage,
    clearMessages,
  };
}
