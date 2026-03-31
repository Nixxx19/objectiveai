package tests

import (
	"encoding/json"
	"path/filepath"
	"testing"

	. "github.com/objective-ai/objectiveai-go/objectiveai"
)

func TestFunctionsInventionsHTTP(t *testing.T) {
	c := getTestClient(t)
	snapshotsDir := filepath.Join(assetsDir(), "functions", "inventions", "client_tests")

	mockInventionAgent := map[string]any{"upstream": "mock", "output_mode": "instruction", "invention": true}

	// json.RawMessage preserves key order to match JS/Python request bodies.
	// Go's map[string]any alphabetizes keys, which changes the mock server's
	// seeded random output.
	cases := []httpTestCase{
		{
			Snapshot: "scalar_leaf_s42_0",
			Body: map[string]any{
				"state": json.RawMessage(`{
					"type": "alpha.scalar.leaf.function",
					"depth": 0, "min_branch_width": 3, "max_branch_width": 5,
					"min_leaf_width": 3, "max_leaf_width": 5,
					"name": "sl-default",
					"spec": "Test function spec for mock invention."
				}`),
				"agent":            mockInventionAgent,
				"seed":             42,
				"stream":           true,
				"max_step_retries": 1,
			},
		},
		{
			Snapshot: "vector_branch_s2025_0",
			Body: map[string]any{
				"state": json.RawMessage(`{
					"type": "alpha.vector.branch.function",
					"depth": 3, "min_branch_width": 2, "max_branch_width": 4,
					"min_leaf_width": 2, "max_leaf_width": 4,
					"name": "vb-deep",
					"spec": "Test function spec for mock invention."
				}`),
				"agent":            mockInventionAgent,
				"seed":             2025,
				"stream":           true,
				"max_step_retries": 1,
			},
		},
		{
			Snapshot: "scalar_leaf_schema_kitchen_0",
			Body: map[string]any{
				"state": json.RawMessage(`{
					"type": "alpha.scalar.leaf.function",
					"depth": 0, "min_branch_width": 3, "max_branch_width": 5,
					"min_leaf_width": 3, "max_leaf_width": 5,
					"name": "sl-kitchen",
					"spec": "Test function spec for mock invention.",
					"input_schema": {
						"type": "object",
						"properties": {
							"name": {"type": "string"},
							"age": {"type": "integer"},
							"score": {"type": "number"},
							"active": {"type": "boolean"},
							"avatar": {"type": "image"},
							"voicemail": {"type": "audio"},
							"demo": {"type": "video"},
							"resume": {"type": "file"},
							"aliases": {
								"type": "array",
								"items": {"anyOf": [{"type": "string"}, {"type": "integer"}]},
								"minItems": 1,
								"maxItems": 8
							},
							"extra": {
								"anyOf": [
									{"type": "string"},
									{
										"type": "array",
										"items": {
											"type": "object",
											"properties": {
												"key": {"type": "string"},
												"val": {"anyOf": [{"type": "number"}, {"type": "boolean"}, {"type": "image"}]}
											},
											"required": ["key", "val"]
										},
										"minItems": 1,
										"maxItems": 3
									}
								]
							}
						},
						"required": ["name", "age", "score", "active", "avatar", "voicemail", "demo", "resume", "aliases", "extra"]
					}
				}`),
				"agent":            mockInventionAgent,
				"seed":             80004,
				"stream":           true,
				"max_step_retries": 1,
			},
		},
	}

	for _, tc := range cases {
		t.Run(tc.Snapshot+"/unary", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runUnary[FunctionsInventionsResponseUnaryFunctionInvention](t, c, "/functions/inventions", tc.Body)
			normalized, err := NormalizeFunctionInventionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})

		t.Run(tc.Snapshot+"/streaming", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runStreaming(t, c, "/functions/inventions", tc.Body,
				func(acc, chunk *FunctionsInventionsResponseStreamingFunctionInventionChunk) {
					acc.Push(chunk)
				},
				FunctionInventionChunkToUnary,
			)
			normalized, err := NormalizeFunctionInventionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})
	}
}
