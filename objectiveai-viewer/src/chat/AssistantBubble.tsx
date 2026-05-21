import cn from "classnames";
import type {
  AgentCompletionsMessageRichContentPart,
  AgentCompletionsResponseStreamingAssistantResponseChunk,
} from "@objectiveai/sdk";

interface AssistantBubbleProps {
  msg: AgentCompletionsResponseStreamingAssistantResponseChunk;
  streaming?: boolean;
}

export function AssistantBubble({ msg, streaming = false }: AssistantBubbleProps) {
  const pulseHere = streaming && !msg.finish_reason;
  return (
    <div
      className={cn(
        "self-start",
        "max-w-[85%]",
        "rounded-2xl",
        "rounded-bl-sm",
        "bg-white",
        "dark:bg-neutral-800",
        "text-neutral-900",
        "dark:text-neutral-50",
        "px-3",
        "py-2",
        "text-sm",
        "shadow-sm",
        "border",
        "border-neutral-200",
        "dark:border-neutral-700",
        "whitespace-pre-wrap",
        "break-words",
      )}
    >
      {msg.reasoning && (
        <details
          className={cn(
            "mb-2",
            "rounded",
            "bg-neutral-100",
            "dark:bg-neutral-900",
            "px-2",
            "py-1",
          )}
        >
          <summary
            className={cn(
              "cursor-pointer",
              "text-xs",
              "text-neutral-500",
              "dark:text-neutral-400",
              "select-none",
            )}
          >
            Thinking
          </summary>
          <div
            className={cn(
              "mt-1",
              "text-xs",
              "text-neutral-600",
              "dark:text-neutral-400",
              "whitespace-pre-wrap",
            )}
          >
            {msg.reasoning}
          </div>
        </details>
      )}

      {msg.refusal && (
        <div
          className={cn(
            "rounded",
            "bg-amber-50",
            "dark:bg-amber-950",
            "text-amber-800",
            "dark:text-amber-200",
            "border",
            "border-amber-200",
            "dark:border-amber-800",
            "px-2",
            "py-1",
            "text-xs",
            "mb-2",
          )}
        >
          {msg.refusal}
        </div>
      )}

      {msg.content !== null && msg.content !== undefined && (
        <RichContent content={msg.content as unknown} />
      )}

      {pulseHere && (
        <span
          className={cn(
            "inline-block",
            "ml-1",
            "w-1.5",
            "h-3",
            "bg-neutral-400",
            "dark:bg-neutral-500",
            "align-middle",
            "animate-pulse",
          )}
          aria-label="streaming"
        />
      )}
    </div>
  );
}

function RichContent({ content }: { content: unknown }) {
  if (typeof content === "string") {
    return <>{content}</>;
  }
  if (Array.isArray(content)) {
    return (
      <>
        {content.map((part, i) => (
          <RichContentPart key={i} part={part as AgentCompletionsMessageRichContentPart} />
        ))}
      </>
    );
  }
  return null;
}

function RichContentPart({ part }: { part: AgentCompletionsMessageRichContentPart }) {
  if ("text" in part && typeof part.text === "string") {
    return <span>{part.text}</span>;
  }
  if ("image_url" in part) {
    const url = (part as { image_url: { url: string } }).image_url.url;
    return (
      <img
        src={url}
        alt=""
        className={cn(
          "max-w-full",
          "max-h-48",
          "object-contain",
          "rounded-md",
          "border",
          "border-neutral-200",
          "dark:border-neutral-700",
        )}
      />
    );
  }
  return null;
}
