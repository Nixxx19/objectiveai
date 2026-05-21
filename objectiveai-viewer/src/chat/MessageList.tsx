import cn from "classnames";
import type {
  AgentCompletionsResponseStreamingAssistantResponseChunk,
} from "@objectiveai/sdk";
import type { PanelTab, PanelTabCompletionEntry } from "../RightOverlayPanel";
import { UserBubble } from "./UserBubble";
import { AssistantBubble } from "./AssistantBubble";

interface MessageListProps {
  tab: PanelTab;
}

export function MessageList({ tab }: MessageListProps) {
  if (tab.entries.length === 0) {
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
      {tab.entries.map((entry, i) => {
        if (entry.kind === "user") {
          return <UserBubble key={i} msg={entry.message} />;
        }
        return <CompletionBlock key={i} entry={entry} />;
      })}
    </div>
  );
}

function CompletionBlock({ entry }: { entry: PanelTabCompletionEntry }) {
  return (
    <div className={cn("flex", "flex-col", "gap-3")}>
      {entry.chunk?.messages.map((m, mi) => {
        if (m.role !== "assistant") return null;
        return (
          <AssistantBubble
            key={mi}
            msg={m as AgentCompletionsResponseStreamingAssistantResponseChunk}
            streaming={entry.streaming}
          />
        );
      })}

      {entry.streaming && entry.chunk === null && (
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

      {entry.error && (
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
          Error {entry.error.code}: {JSON.stringify(entry.error.message)}
        </div>
      )}
    </div>
  );
}
