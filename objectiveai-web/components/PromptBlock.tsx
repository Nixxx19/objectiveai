"use client";

import { useState, useCallback } from "react";

const COMMAND = "$ npx objectiveai";
const PROMPT = "> Build a function to score resume quality\n  with multi-model validation";

interface PromptBlockProps {
  variant?: "default" | "compact";
}

export default function PromptBlock({ variant = "default" }: PromptBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    const text = `${COMMAND}\n${PROMPT}`;
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = text;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      document.body.removeChild(textarea);
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }, []);

  if (variant === "compact") {
    return (
      <div className="promptBlockCompact">
        <div className="promptBlockBody">
          <code className="promptBlockCommand">{COMMAND}</code>
          <code className="promptBlockPrompt">{PROMPT}</code>
        </div>
        <button
          className={`promptBlockCopy ${copied ? "promptBlockCopyCopied" : ""}`}
          onClick={handleCopy}
        >
          {copied ? "\u2713 Copied" : "Copy"}
        </button>
      </div>
    );
  }

  return (
    <div className="promptBlock">
      <div className="promptBlockHeader">
        <div className="promptBlockDots">
          <span className="dot dotRed" />
          <span className="dot dotYellow" />
          <span className="dot dotGreen" />
        </div>
        <button
          className={`promptBlockCopy ${copied ? "promptBlockCopyCopied" : ""}`}
          onClick={handleCopy}
        >
          {copied ? "\u2713 Copied" : "Copy"}
        </button>
      </div>
      <div className="promptBlockBody">
        <code className="promptBlockCommand">{COMMAND}</code>
        <code className="promptBlockPrompt">{PROMPT}</code>
      </div>
    </div>
  );
}
