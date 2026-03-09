import { describe, it, expect } from "vitest";
import { Functions } from "../../../index.js";

// ── helpers ──────────────────────────────────────────────────────────

function textInputSchema() {
  return {
    type: "object",
    properties: {
      text: { type: "string" },
    },
    required: ["text"],
  };
}

function starlarkMessages(expr: string) {
  return { $starlark: expr };
}

function twoTextResponses(a: string, b: string) {
  return [
    [{ type: "text", text: a }],
    [{ type: "text", text: b }],
  ];
}

function validLeaf() {
  return {
    type: "alpha.scalar.leaf.function",
    description: "test",
    input_schema: textInputSchema(),
    tasks: [
      {
        type: "vector.completion",
        messages: starlarkMessages(
          "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
        ),
        responses: twoTextResponses("Option A", "Option B"),
      },
    ],
  };
}

// ── success tests ────────────────────────────────────────────────────

describe("alphaCheckLeafScalarFunction", () => {
  it("accepts a valid single-task leaf function", () => {
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(validLeaf() as any),
    ).not.toThrow();
  });

  it("accepts multiple tasks", () => {
    const f = {
      ...validLeaf(),
      tasks: [
        {
          type: "vector.completion",
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
          ),
          responses: twoTextResponses("Option A", "Option B"),
        },
        {
          type: "vector.completion",
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': 'Review: ' + input['text']}]}]",
          ),
          responses: twoTextResponses("Good", "Bad"),
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).not.toThrow();
  });

  it("accepts task with skip expression", () => {
    const f = {
      ...validLeaf(),
      input_schema: {
        type: "object",
        properties: {
          text: { type: "string" },
          skip_last_task: { type: "boolean" },
        },
        required: ["text"],
      },
      tasks: [
        {
          type: "vector.completion",
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
          ),
          responses: twoTextResponses("Yes", "No"),
        },
        {
          type: "vector.completion",
          skip: { $starlark: "input.get('skip_last_task', False)" },
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': 'Review: ' + input['text']}]}]",
          ),
          responses: twoTextResponses("Good", "Bad"),
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).not.toThrow();
  });

  // ── error tests ──────────────────────────────────────────────────

  it("rejects wrong type (branch)", () => {
    const f = {
      type: "alpha.scalar.branch.function",
      description: "test",
      input_schema: textInputSchema(),
      tasks: [],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/AS01/);
  });

  it("rejects empty description", () => {
    const f = { ...validLeaf(), description: "  " };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/QD01/);
  });

  it("rejects no tasks", () => {
    const f = { ...validLeaf(), tasks: [] };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/AS03/);
  });

  it("rejects fewer than 2 responses", () => {
    const f = {
      ...validLeaf(),
      tasks: [
        {
          type: "vector.completion",
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
          ),
          responses: [[{ type: "text", text: "Only one" }]],
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/AS10/);
  });

  it("rejects fixed parameters (diversity fail)", () => {
    const f = {
      ...validLeaf(),
      tasks: [
        {
          type: "vector.completion",
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': 'hello'}]}]",
          ),
          responses: twoTextResponses("A", "B"),
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/AS19/);
  });

  it("rejects all tasks skipped", () => {
    const f = {
      ...validLeaf(),
      tasks: [
        {
          type: "vector.completion",
          skip: { $starlark: "True" },
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
          ),
          responses: twoTextResponses("Yes", "No"),
        },
        {
          type: "vector.completion",
          skip: { $starlark: "True" },
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': input['text']}]}]",
          ),
          responses: twoTextResponses("Good", "Bad"),
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/CV42/);
  });

  it("rejects single-permutation input schema (enum with 1 value)", () => {
    const f = {
      ...validLeaf(),
      input_schema: {
        type: "object",
        properties: {
          value: { type: "string", enum: ["only"] },
        },
        required: ["value"],
      },
      tasks: [
        {
          type: "vector.completion",
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': input['value']}]}]",
          ),
          responses: twoTextResponses("yes", "no"),
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/QI01/);
  });

  it("rejects image in schema but not in messages", () => {
    const f = {
      ...validLeaf(),
      input_schema: {
        type: "object",
        properties: {
          photo: { type: "image" },
          label: { type: "string" },
        },
        required: ["photo", "label"],
      },
      tasks: [
        {
          type: "vector.completion",
          messages: starlarkMessages(
            "[{'role': 'user', 'content': [{'type': 'text', 'text': input['label']}]}]",
          ),
          responses: twoTextResponses("good", "bad"),
        },
      ],
    };
    expect(() =>
      Functions.AlphaScalar.Check.alphaCheckLeafScalarFunction(f as any),
    ).toThrow(/AS20/);
  });
});
