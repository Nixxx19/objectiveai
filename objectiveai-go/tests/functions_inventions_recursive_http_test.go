package tests

import (
	"path/filepath"
	"testing"

	. "github.com/objective-ai/objectiveai-go/objectiveai"
)

func TestFunctionsInventionsRecursiveHTTP(t *testing.T) {
	c := getTestClient(t)
	snapshotsDir := filepath.Join(assetsDir(), "functions", "inventions", "recursive_client_tests")

	mockInventionAgent := map[string]any{"upstream": "mock", "output_mode": "instruction", "invention": true}

	starlarkMessages := map[string]any{"$starlark": `[{"role": "user", "content": [{"type": "text", "text": str(input)}]}]`}
	starlarkRankMessages := map[string]any{"$starlark": `[{"role": "user", "content": [{"type": "text", "text": "rank these"}]}]`}
	starlarkRankResponses := map[string]any{"$starlark": `[[{"type": "text", "text": str(item)}] for item in input['items']]`}

	yesNoResponses := []any{
		[]any{map[string]any{"type": "text", "text": "yes"}},
		[]any{map[string]any{"type": "text", "text": "no"}},
	}

	cases := []httpTestCase{
		{
			Snapshot: "valid_schema_valid_tasks_scalar_leaf",
			Body: map[string]any{
				"remote": "mock",
				"state": map[string]any{
					"type": "alpha.scalar.leaf.function",
					"depth": 0, "min_branch_width": 1, "max_branch_width": 1,
					"min_leaf_width": 2, "max_leaf_width": 4,
					"name": "inv-good-sl",
					"spec": "Test function spec for mock recursive invention.",
					"input_schema": map[string]any{
						"type": "object",
						"properties": map[string]any{
							"sentiment": map[string]any{"type": "string", "enum": []any{"positive", "negative"}},
						},
						"required": []any{"sentiment"},
					},
					"essay_tasks": "Good tasks incoming.",
					"tasks": []any{
						map[string]any{
							"type":      "vector.completion",
							"messages":  starlarkMessages,
							"responses": yesNoResponses,
						},
						map[string]any{
							"type":      "vector.completion",
							"messages":  starlarkMessages,
							"responses": yesNoResponses,
						},
					},
					"tasks_length": 2,
					"description":  "A valid scalar function.",
				},
				"agent":            mockInventionAgent,
				"seed":             5300,
				"stream":           true,
				"max_step_retries": 1,
			},
		},
		{
			Snapshot: "valid_vector_schema_valid_tasks",
			Body: map[string]any{
				"remote": "mock",
				"state": map[string]any{
					"type": "alpha.vector.leaf.function",
					"depth": 0, "min_branch_width": 1, "max_branch_width": 1,
					"min_leaf_width": 2, "max_leaf_width": 4,
					"name":  "inv-good-vl",
					"spec":  "Test function spec for mock recursive invention.",
					"essay": "Ranking things.",
					"input_schema": map[string]any{
						"items": map[string]any{"type": "string", "enum": []any{"apple", "banana"}},
					},
					"tasks": []any{
						map[string]any{
							"type":      "vector.completion",
							"messages":  starlarkRankMessages,
							"responses": starlarkRankResponses,
						},
						map[string]any{
							"type":      "vector.completion",
							"messages":  starlarkRankMessages,
							"responses": starlarkRankResponses,
						},
					},
					"tasks_length": 2,
				},
				"agent":            mockInventionAgent,
				"seed":             5400,
				"stream":           true,
				"max_step_retries": 1,
			},
		},
		{
			Snapshot: "valid_schema_no_tasks_with_essay",
			Body: map[string]any{
				"remote": "mock",
				"state": map[string]any{
					"type": "alpha.scalar.leaf.function",
					"depth": 0, "min_branch_width": 1, "max_branch_width": 1,
					"min_leaf_width": 2, "max_leaf_width": 4,
					"name":  "inv-schema-only",
					"spec":  "Test function spec for mock recursive invention.",
					"essay": "A great essay about things.",
					"input_schema": map[string]any{
						"type": "object",
						"properties": map[string]any{
							"sentiment": map[string]any{"type": "string", "enum": []any{"positive", "negative"}},
						},
						"required": []any{"sentiment"},
					},
				},
				"agent":            mockInventionAgent,
				"seed":             5900,
				"stream":           true,
				"max_step_retries": 1,
			},
		},
	}

	for _, tc := range cases {
		t.Run(tc.Snapshot+"/unary", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runUnary[FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursive](t, c, "/functions/inventions/recursive", tc.Body)
			normalized, err := NormalizeFunctionInventionRecursiveForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})

		t.Run(tc.Snapshot+"/streaming", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runStreaming(t, c, "/functions/inventions/recursive", tc.Body,
				func(acc, chunk *FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk) {
					acc.Push(chunk)
				},
				FunctionInventionRecursiveChunkToUnary,
			)
			normalized, err := NormalizeFunctionInventionRecursiveForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})
	}
}
