package tests

import (
	"path/filepath"
	"testing"

	. "github.com/objective-ai/objectiveai-go/objectiveai"
)

func TestFunctionsExecutionsHTTP(t *testing.T) {
	c := getTestClient(t)
	snapshotsDir := filepath.Join(assetsDir(), "functions", "executions", "client_tests")

	mockRemote := func(name string) map[string]any {
		return map[string]any{"remote": "mock", "name": name}
	}

	cases := []httpTestCase{
		{
			Snapshot: "mock_1_scalar_leaf_binary_seed_42",
			Body: map[string]any{
				"function": mockRemote("binary-classifier"),
				"profile":  mockRemote("solo-instruction"),
				"input":    map[string]any{"text": "Hello world"},
				"seed":     42,
			},
		},
		{
			Snapshot: "mock_7_vector_5_criteria_seed_42",
			Body: map[string]any{
				"function": mockRemote("five-criteria-ranker"),
				"profile":  mockRemote("schema-heavy-trio"),
				"input":    map[string]any{"items": []any{"Option A", "Option B", "Option C"}},
				"seed":     42,
			},
		},
		{
			Snapshot: "mock_20_vector_super_branch_seed_42",
			Body: map[string]any{
				"function": mockRemote("nested-vector-super-branch"),
				"profile":  mockRemote("nested-vector-inline-remote"),
				"input":    map[string]any{"items": []any{"Alpha", "Beta", "Gamma"}},
				"seed":     42,
			},
		},
	}

	for _, tc := range cases {
		t.Run(tc.Snapshot+"/unary", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runUnary[FunctionsExecutionsResponseUnaryFunctionExecution](t, c, "/functions/executions", tc.Body)
			normalized, err := NormalizeFunctionExecutionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})

		t.Run(tc.Snapshot+"/streaming", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runStreaming(t, c, "/functions/executions", tc.Body,
				func(acc, chunk *FunctionsExecutionsResponseStreamingFunctionExecutionChunk) {
					acc.Push(chunk)
				},
				FunctionExecutionChunkToUnary,
			)
			normalized, err := NormalizeFunctionExecutionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})
	}
}
