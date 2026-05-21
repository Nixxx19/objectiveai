import cn from "classnames";
import type {
  AgentCompletionsResponseStreamingAssistantResponseChunk,
} from "@objectiveai/sdk";
import type { PanelTab } from "../RightOverlayPanel";
import { UserBubble } from "./UserBubble";
import { AssistantBubble } from "./AssistantBubble";

interface MessageListProps {
  tab: PanelTab;
}

export function MessageList({ tab }: MessageListProps) {
  if (tab.turns.length === 0) {
    return (
      <div
        className={cn(
          "flex",
          "flex-1",
          "items-center",
          "justify-center",
          "text-sm",
          "text-neutral-500",
          "dark:text-neutral-400",
          "italic",
        )}
      >
        Say hi to {tab.favorite.name}.
      </div>
    );
  }

  return (
    <div className={cn("flex", "flex-col", "gap-3", "px-3", "py-4")}>
      {tab.turns.map((turn, ti) => (
        <div key={ti} className={cn("flex", "flex-col", "gap-3")}>
          {turn.users.map((u, ui) => (
            <UserBubble key={`u-${ui}`} msg={u} />
          ))}

          {turn.completion?.messages.map((m, mi) => {
            if (m.role !== "assistant") return null;
            return (
              <AssistantBubble
                key={`a-${mi}`}
                msg={m as AgentCompletionsResponseStreamingAssistantResponseChunk}
                streaming={turn.streaming}
              />
            );
          })}

          {turn.streaming && turn.completion === null && (
            <div
              className={cn(
                "self-start",
                "text-xs",
                "italic",
                "text-neutral-500",
                "dark:text-neutral-400",
              )}
            >
              …
            </div>
          )}

          {turn.error && (
            <div
              className={cn(
                "self-stretch",
                "rounded",
                "border",
                "border-red-200",
                "dark:border-red-800",
                "bg-red-50",
                "dark:bg-red-950",
                "text-red-700",
                "dark:text-red-300",
                "px-3",
                "py-2",
                "text-xs",
              )}
            >
              Error {turn.error.code}: {JSON.stringify(turn.error.message)}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
