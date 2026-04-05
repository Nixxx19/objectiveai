import type { FunctionDef } from "./types";

/**
 * Real mock functions from the ObjectiveAI API binary.
 * These are the exact structures — nothing invented.
 */

export const binaryClassifier: FunctionDef = {
  type: "alpha.scalar.leaf.function",
  description: "Classifies text as yes or no.",
  input_schema: {
    type: "object",
    properties: { text: { type: "string", description: "Text to classify" } },
    required: ["text"],
  },
  tasks: [
    {
      type: "vector.completion",
      messages: {
        $starlark:
          "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
      },
      responses: [
        [{ type: "text", text: "Yes" }],
        [{ type: "text", text: "No" }],
      ],
    },
  ],
};

export const emailImportance: FunctionDef = {
  type: "alpha.scalar.leaf.function",
  description: "Rates how important an email is on a five-point scale.",
  input_schema: {
    type: "object",
    properties: {
      subject: { type: "string" },
      body: { type: "string" },
    },
    required: ["subject", "body"],
  },
  tasks: [
    {
      type: "vector.completion",
      messages: {
        $starlark:
          "[{'role': 'user', 'content': [{'type': 'text', 'text': 'Rate importance: ' + input['subject'] + ' — ' + input['body']}]}]",
      },
      responses: [
        [{ type: "text", text: "Critical" }],
        [{ type: "text", text: "Important" }],
        [{ type: "text", text: "Normal" }],
        [{ type: "text", text: "Low" }],
        [{ type: "text", text: "Ignore" }],
      ],
    },
  ],
};

export const fiveStarRating: FunctionDef = {
  type: "alpha.scalar.leaf.function",
  description: "Rates text on a 1-5 star scale.",
  input_schema: {
    type: "object",
    properties: { text: { type: "string" } },
    required: ["text"],
  },
  tasks: [
    {
      type: "vector.completion",
      messages: {
        $starlark:
          "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
      },
      responses: [
        [{ type: "text", text: "★★★★★" }],
        [{ type: "text", text: "★★★★" }],
        [{ type: "text", text: "★★★" }],
        [{ type: "text", text: "★★" }],
        [{ type: "text", text: "★" }],
      ],
    },
  ],
};

export const sentimentClassifier: FunctionDef = {
  type: "alpha.scalar.leaf.function",
  description: "Classifies text sentiment as positive, negative, or neutral.",
  input_schema: {
    type: "object",
    properties: { text: { type: "string" } },
    required: ["text"],
  },
  tasks: [
    {
      type: "vector.completion",
      messages: {
        $starlark:
          "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
      },
      responses: [
        [{ type: "text", text: "Positive" }],
        [{ type: "text", text: "Negative" }],
        [{ type: "text", text: "Neutral" }],
      ],
    },
  ],
};

export const spamImportanceBranch: FunctionDef = {
  type: "alpha.scalar.branch.function",
  description:
    "Evaluates text for both spam likelihood and email importance.",
  input_schema: {
    type: "object",
    properties: {
      text: { type: "string" },
      subject: { type: "string" },
    },
    required: ["text", "subject"],
  },
  tasks: [
    {
      type: "alpha.scalar.function",
      remote: "mock",
      name: "binary-classifier",
      input: { $starlark: "{'text': input['text']}" },
    },
    {
      type: "alpha.scalar.function",
      remote: "mock",
      name: "email-importance",
      input: {
        $starlark:
          "{'subject': input['subject'], 'body': input['text']}",
      },
    },
  ],
};

export const tripleClassifierBranch: FunctionDef = {
  type: "alpha.scalar.branch.function",
  description:
    "Combines binary, five-star, and sentiment classifiers for comprehensive text analysis.",
  input_schema: {
    type: "object",
    properties: { text: { type: "string" } },
    required: ["text"],
  },
  tasks: [
    {
      type: "alpha.scalar.function",
      remote: "mock",
      name: "binary-classifier",
      input: { $starlark: "{'text': input['text']}" },
    },
    {
      type: "alpha.scalar.function",
      remote: "mock",
      name: "five-star-rating",
      input: { $starlark: "{'text': input['text']}" },
    },
    {
      type: "alpha.scalar.function",
      remote: "mock",
      name: "sentiment-classifier",
      input: { $starlark: "{'text': input['text']}" },
    },
  ],
};

export const nestedScalarSuperBranch: FunctionDef = {
  type: "alpha.scalar.branch.function",
  description:
    "Two-level nested text evaluation combining the spam-importance and triple-classifier branches.",
  input_schema: {
    type: "object",
    properties: {
      text: { type: "string" },
      subject: { type: "string" },
    },
    required: ["text", "subject"],
  },
  tasks: [
    {
      type: "alpha.scalar.function",
      remote: "mock",
      name: "spam-importance-branch",
      input: {
        $starlark:
          "{'text': input['text'], 'subject': input['subject']}",
      },
    },
    {
      type: "alpha.scalar.function",
      remote: "mock",
      name: "triple-classifier-branch",
      input: { $starlark: "{'text': input['text']}" },
    },
  ],
};

/** Build the registry of all resolvable functions */
export function buildMockRegistry(): Map<string, FunctionDef> {
  const reg = new Map<string, FunctionDef>();
  reg.set("binary-classifier", binaryClassifier);
  reg.set("mock/binary-classifier", binaryClassifier);
  reg.set("email-importance", emailImportance);
  reg.set("mock/email-importance", emailImportance);
  reg.set("five-star-rating", fiveStarRating);
  reg.set("mock/five-star-rating", fiveStarRating);
  reg.set("sentiment-classifier", sentimentClassifier);
  reg.set("mock/sentiment-classifier", sentimentClassifier);
  reg.set("spam-importance-branch", spamImportanceBranch);
  reg.set("mock/spam-importance-branch", spamImportanceBranch);
  reg.set("triple-classifier-branch", tripleClassifierBranch);
  reg.set("mock/triple-classifier-branch", tripleClassifierBranch);
  return reg;
}
