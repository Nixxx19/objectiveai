import { describe, it, expect } from "vitest";
import { Functions } from "../../../index.js";

// ── helpers ──────────────────────────────────────────────────────────

function arrayItemsInputSchema() {
  return {
    items: {
      type: "array",
      items: { type: "string" },
      minItems: 2,
      maxItems: 5,
    },
  };
}

function validLeaf() {
  return {
    type: "alpha.vector.leaf.function",
    description: "test",
    input_schema: arrayItemsInputSchema(),
    tasks: [
      {
        type: "vector.completion",
        messages: {
          $starlark:
            "[{'role': 'user', 'content': [{'type': 'text', 'text': ', '.join(input['items'])}]}]",
        },
        responses: {
          $starlark:
            "[[{'type': 'text', 'text': x}] for x in input['items']]",
        },
      },
    ],
  };
}

// ── success tests ────────────────────────────────────────────────────

describe("alphaCheckLeafVectorFunction", () => {
  it("accepts a valid single-task leaf function", () => {
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(
        validLeaf() as any,
      ),
    ).not.toThrow();
  });

  it("accepts multiple tasks", () => {
    const f = {
      ...validLeaf(),
      tasks: [
        {
          type: "vector.completion",
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': ', '.join(input['items'])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': x}] for x in input['items']]",
          },
        },
        {
          type: "vector.completion",
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': 'Rank: ' + ', '.join(input['items'])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': 'item: ' + x}] for x in input['items']]",
          },
        },
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).not.toThrow();
  });

  it("accepts with context in input schema", () => {
    const f = {
      type: "alpha.vector.leaf.function",
      description: "test",
      input_schema: {
        context: {
          type: "object",
          properties: {
            topic: { type: "string" },
          },
          required: ["topic"],
        },
        items: {
          type: "array",
          items: { type: "string" },
          minItems: 2,
          maxItems: 5,
        },
      },
      tasks: [
        {
          type: "vector.completion",
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': input['context']['topic'] + ': ' + ', '.join(input['items'])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': x}] for x in input['items']]",
          },
        },
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).not.toThrow();
  });

  it("accepts with skip expression", () => {
    const f = {
      ...validLeaf(),
      input_schema: {
        items: {
          type: "array",
          items: {
            type: "object",
            properties: {
              text: { type: "string" },
              skip_extra: { type: "boolean" },
            },
            required: ["text", "skip_extra"],
          },
          minItems: 2,
          maxItems: 5,
        },
      },
      tasks: [
        {
          type: "vector.completion",
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': ', '.join([x['text'] for x in input['items']])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': x['text']}] for x in input['items']]",
          },
        },
        {
          type: "vector.completion",
          skip: { $starlark: "input['items'][0]['skip_extra']" },
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': 'Review: ' + ', '.join([x['text'] for x in input['items']])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': 'r: ' + x['text']}] for x in input['items']]",
          },
        },
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).not.toThrow();
  });

  // ── error tests ──────────────────────────────────────────────────

  it("rejects wrong type (branch)", () => {
    const f = {
      type: "alpha.vector.branch.function",
      description: "test",
      input_schema: arrayItemsInputSchema(),
      tasks: [],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).toThrow(/AV01/);
  });

  it("rejects empty description", () => {
    const f = { ...validLeaf(), description: "  " };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).toThrow(/QD01/);
  });

  it("rejects no tasks", () => {
    const f = { ...validLeaf(), tasks: [] };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).toThrow(/AV03/);
  });

  it("rejects fixed response values (AV16)", () => {
    const f = {
      ...validLeaf(),
      tasks: [
        {
          type: "vector.completion",
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': ', '.join(input['items'])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': 'option ' + str(i)}] for i in range(len(input['items']))]",
          },
        },
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).toThrow(/AV16/);
  });

  it("rejects all tasks skipped", () => {
    const f = {
      ...validLeaf(),
      tasks: [
        {
          type: "vector.completion",
          skip: { $starlark: "True" },
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': ', '.join(input['items'])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': x}] for x in input['items']]",
          },
        },
        {
          type: "vector.completion",
          skip: { $starlark: "True" },
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': ', '.join(input['items'])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': x}] for x in input['items']]",
          },
        },
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).toThrow(/CV42/);
  });

  it("rejects output_length < 2 from minItems: 1", () => {
    const f = {
      ...validLeaf(),
      input_schema: {
        items: {
          type: "array",
          items: { type: "string" },
          minItems: 1,
          maxItems: 1,
        },
      },
      tasks: [
        {
          type: "vector.completion",
          messages: {
            $starlark:
              "[{'role': 'user', 'content': [{'type': 'text', 'text': ', '.join(input['items'])}]}]",
          },
          responses: {
            $starlark:
              "[[{'type': 'text', 'text': x}] for x in input['items']]",
          },
        },
      ],
    };
    expect(() =>
      Functions.AlphaVector.Check.alphaCheckLeafVectorFunction(f as any),
    ).toThrow(/VF03/);
  });
});
