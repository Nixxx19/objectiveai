package tests

import (
	"encoding/json"
	"path/filepath"
	"testing"

	. "github.com/objective-ai/objectiveai-go/objectiveai"
)

func TestFunctionsInventionsRecursiveHTTP(t *testing.T) {
	c := getTestClient(t)
	snapshotsDir := filepath.Join(assetsDir(), "functions", "inventions", "recursive_client_tests")

	mockInventionAgent := map[string]any{"upstream": "mock", "output_mode": "instruction", "invention": true}

	// json.RawMessage preserves key order to match JS/Python request bodies.
	cases := []httpTestCase{
		{
			Snapshot: "valid_schema_valid_tasks_scalar_leaf",
			Body: map[string]any{
				"remote": "mock",
				"state": json.RawMessage(`{
					"type": "alpha.scalar.leaf.function",
					"depth": 0, "min_branch_width": 1, "max_branch_width": 1,
					"min_leaf_width": 2, "max_leaf_width": 4,
					"name": "inv-good-sl",
					"spec": "Test function spec for mock recursive invention.",
					"input_schema": {
						"type": "object",
						"properties": {
							"sentiment": {"type": "string", "enum": ["positive", "negative"]}
						},
						"required": ["sentiment"]
					},
					"essay_tasks": "Good tasks incoming.",
					"tasks": [
						{
							"type": "vector.completion",
							"messages": {"$starlark": "[{\"role\": \"user\", \"content\": [{\"type\": \"text\", \"text\": str(input)}]}]"},
							"responses": [[{"type": "text", "text": "yes"}], [{"type": "text", "text": "no"}]]
						},
						{
							"type": "vector.completion",
							"messages": {"$starlark": "[{\"role\": \"user\", \"content\": [{\"type\": \"text\", \"text\": str(input)}]}]"},
							"responses": [[{"type": "text", "text": "yes"}], [{"type": "text", "text": "no"}]]
						}
					],
					"tasks_length": 2,
					"description": "A valid scalar function."
				}`),
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
				"state": json.RawMessage(`{
					"type": "alpha.vector.leaf.function",
					"depth": 0, "min_branch_width": 1, "max_branch_width": 1,
					"min_leaf_width": 2, "max_leaf_width": 4,
					"name": "inv-good-vl",
					"spec": "Test function spec for mock recursive invention.",
					"essay": "Ranking things.",
					"input_schema": {
						"items": {"type": "string", "enum": ["apple", "banana"]}
					},
					"tasks": [
						{
							"type": "vector.completion",
							"messages": {"$starlark": "[{\"role\": \"user\", \"content\": [{\"type\": \"text\", \"text\": \"rank these\"}]}]"},
							"responses": {"$starlark": "[[{\"type\": \"text\", \"text\": str(item)}] for item in input['items']]"}
						},
						{
							"type": "vector.completion",
							"messages": {"$starlark": "[{\"role\": \"user\", \"content\": [{\"type\": \"text\", \"text\": \"rank these\"}]}]"},
							"responses": {"$starlark": "[[{\"type\": \"text\", \"text\": str(item)}] for item in input['items']]"}
						}
					],
					"tasks_length": 2
				}`),
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
				"state": json.RawMessage(`{
					"type": "alpha.scalar.leaf.function",
					"depth": 0, "min_branch_width": 1, "max_branch_width": 1,
					"min_leaf_width": 2, "max_leaf_width": 4,
					"name": "inv-schema-only",
					"spec": "Test function spec for mock recursive invention.",
					"essay": "A great essay about things.",
					"input_schema": {
						"type": "object",
						"properties": {
							"sentiment": {"type": "string", "enum": ["positive", "negative"]}
						},
						"required": ["sentiment"]
					}
				}`),
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
