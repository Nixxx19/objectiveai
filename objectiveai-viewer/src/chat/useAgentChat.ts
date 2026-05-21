import { useCallback, type Dispatch, type SetStateAction } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  AgentCompletionsResponseStreamingAgentCompletionChunkSchema,
  ErrorResponseErrorSchema,
  ViewerEventSchema,
  agentCompletionsResponseStreamingAgentCompletionChunkMerged,
  type AgentCompletionsMessageMessage,
  type AgentCompletionsMessageRichContent,
  type AgentCompletionsMessageRichContentPart,
  type ViewerEvent,
} from "@objectiveai/sdk";
import { buildAgentCompletionRequest } from "./buildAgentCompletionRequest";
import type { PanelTab, PanelTabTurn } from "../RightOverlayPanel";

interface UseAgentChat {
  sendMessage: (tabId: string) => void;
}

export function useAgentChat(
  setPanelTabs: Dispatch<SetStateAction<PanelTab[]>>,
): UseAgentChat {
  const fireNotify = useCallback(
    async (responseId: string, content: AgentCompletionsMessageRichContent) => {
      const params = { response_id: responseId, content };
      const origin = `agent-completion-notify-${crypto.randomUUID()}`;
      let unlisten: UnlistenFn | undefined;
      unlisten = await listen<ViewerEvent>(origin, (ev) => {
        const ce = ViewerEventSchema.safeParse(ev.payload);
        if (!ce.success || ce.data.type !== "cli_command") return;
        const line = ce.data.value as { type?: string };
        if (line?.type === "end") unlisten?.();
      });
      invoke("cli_run", {
        args: [
          "api", "agent", "completions", "notify", "post",
          "--body-inline", JSON.stringify(params),
        ],
        origin,
      });
    },
    [],
  );

  const finalizeTurn = useCallback(
    (tabId: string) => {
      let drain = false;
      setPanelTabs((prev) => prev.map((t) => {
        if (t.id !== tabId || t.inFlightIndex === null) return t;
        const idx = t.inFlightIndex;
        const turn = t.turns[idx];
        if (!turn) return t;
        const finalChunk = turn.completion;
        const continuation = finalChunk?.continuation ?? t.continuation;
        const messagesQueued = finalChunk?.messages_queued === true;
        const turns = t.turns.slice();
        turns[idx] = { ...turn, streaming: false };
        drain = messagesQueued;
        return {
          ...t,
          turns,
          continuation,
          inFlightIndex: null,
          pendingNotifyContent: [],
        };
      }));
      if (drain) {
        queueMicrotask(() => startTurn(tabId, []));
      }
    },
    [setPanelTabs],
  );

  const startTurn = useCallback(
    async (tabId: string, users: AgentCompletionsMessageMessage[]) => {
      const requestId = crypto.randomUUID();
      const origin = `agent-completion-${requestId}`;

      let favorite: PanelTab["favorite"] | undefined;
      let continuation: string | null = null;
      let invalid = false;
      setPanelTabs((prev) => prev.map((t) => {
        if (t.id !== tabId) return t;
        if (t.inFlightIndex !== null) {
          invalid = true;
          return t;
        }
        favorite = t.favorite;
        continuation = t.continuation;
        const turn: PanelTabTurn = {
          users,
          completion: null,
          streaming: true,
          error: null,
          requestId,
        };
        return {
          ...t,
          turns: [...t.turns, turn],
          inFlightIndex: t.turns.length,
          pendingNotifyContent: [],
        };
      }));
      if (invalid || !favorite) return;

      const request = buildAgentCompletionRequest(favorite, users, continuation);

      let unlisten: UnlistenFn | undefined;
      unlisten = await listen<ViewerEvent>(origin, (ev) => {
        const ce = ViewerEventSchema.safeParse(ev.payload);
        if (!ce.success || ce.data.type !== "cli_command") return;
        const line = ce.data.value as { type?: string; value?: unknown };
        if (line.type === "notification") {
          const parsed = AgentCompletionsResponseStreamingAgentCompletionChunkSchema
            .safeParse(line.value);
          if (!parsed.success) return;
          const next = parsed.data;
          const toFlush: AgentCompletionsMessageRichContent[] = [];
          let firstChunkResponseId: string | null = null;
          setPanelTabs((prev) => prev.map((t) => {
            if (t.id !== tabId || t.inFlightIndex === null) return t;
            const idx = t.inFlightIndex;
            const turn = t.turns[idx];
            if (!turn || turn.requestId !== requestId) return t;
            const wasNull = turn.completion === null;
            const merged: typeof next = turn.completion
              ? agentCompletionsResponseStreamingAgentCompletionChunkMerged(
                  turn.completion, next,
                )[0]
              : next;
            if (wasNull && merged.id && t.pendingNotifyContent.length > 0) {
              firstChunkResponseId = merged.id;
              toFlush.push(...t.pendingNotifyContent);
            }
            const turns = t.turns.slice();
            turns[idx] = { ...turn, completion: merged };
            return wasNull && toFlush.length > 0
              ? { ...t, turns, pendingNotifyContent: [] }
              : { ...t, turns };
          }));
          if (firstChunkResponseId !== null) {
            const rid = firstChunkResponseId;
            for (const content of toFlush) {
              queueMicrotask(() => fireNotify(rid, content));
            }
          }
        } else if (line.type === "error") {
          const parsed = ErrorResponseErrorSchema.safeParse(line.value);
          if (!parsed.success) return;
          setPanelTabs((prev) => prev.map((t) => {
            if (t.id !== tabId || t.inFlightIndex === null) return t;
            const idx = t.inFlightIndex;
            const turn = t.turns[idx];
            if (!turn || turn.requestId !== requestId) return t;
            const turns = t.turns.slice();
            turns[idx] = { ...turn, error: parsed.data };
            return { ...t, turns };
          }));
        } else if (line.type === "end") {
          unlisten?.();
          finalizeTurn(tabId);
        }
      });

      invoke("cli_run", {
        args: [
          "api", "agent", "completions", "post",
          "--body-inline", JSON.stringify(request),
        ],
        origin,
      });
    },
    [setPanelTabs, fireNotify, finalizeTurn],
  );

  const sendMessage = useCallback(
    (tabId: string) => {
      let userMsg: AgentCompletionsMessageMessage | null = null;
      let startFresh = false;
      let notifyNow: { responseId: string; content: AgentCompletionsMessageRichContent } | null = null;
      setPanelTabs((prev) => prev.map((t) => {
        if (t.id !== tabId) return t;
        const built = buildUserMessage(t.draft, t.attachments);
        if (built === null) return t;
        userMsg = built;
        const cleared = { ...t, draft: "", attachments: [] };
        if (t.inFlightIndex === null) {
          startFresh = true;
          return cleared;
        }
        const idx = t.inFlightIndex;
        const turn = t.turns[idx];
        if (!turn) return cleared;
        const turns = t.turns.slice();
        turns[idx] = { ...turn, users: [...turn.users, built] };
        const content = (built as { content: AgentCompletionsMessageRichContent }).content;
        if (turn.completion?.id) {
          notifyNow = { responseId: turn.completion.id, content };
          return { ...cleared, turns };
        }
        return {
          ...cleared,
          turns,
          pendingNotifyContent: [...t.pendingNotifyContent, content],
        };
      }));
      if (startFresh && userMsg) {
        const msg = userMsg;
        queueMicrotask(() => startTurn(tabId, [msg]));
      }
      if (notifyNow) {
        const n = notifyNow as { responseId: string; content: AgentCompletionsMessageRichContent };
        queueMicrotask(() => fireNotify(n.responseId, n.content));
      }
    },
    [setPanelTabs, startTurn, fireNotify],
  );

  return { sendMessage };
}

function buildUserMessage(
  draft: string,
  attachments: AgentCompletionsMessageRichContentPart[],
): AgentCompletionsMessageMessage | null {
  const parts: AgentCompletionsMessageRichContentPart[] = [];
  if (draft.trim().length > 0) parts.push({ type: "text", text: draft });
  parts.push(...attachments);
  if (parts.length === 0) return null;
  return { role: "user", content: parts } as AgentCompletionsMessageMessage;
}
