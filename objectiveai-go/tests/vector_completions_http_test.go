package tests

import (
	"fmt"
	"path/filepath"
	"testing"

	. "github.com/objective-ai/objectiveai-go/objectiveai"
)

func TestVectorCompletionsHTTP(t *testing.T) {
	c := getTestClient(t)
	snapshotsDir := filepath.Join(assetsDir(), "vector", "completions", "client_tests")

	mockAgent := map[string]any{"upstream": "mock", "output_mode": "instruction"}

	responses25 := make([]any, 25)
	for i := range responses25 {
		responses25[i] = fmt.Sprintf("Response %d", i)
	}

	cases := []httpTestCase{
		{
			Snapshot: "single_agent_2_responses_instruction_seed_42",
			Body: map[string]any{
				"messages":  []any{map[string]any{"role": "user", "content": "Which is better?"}},
				"swarm":     map[string]any{"agents": []any{mockAgent}},
				"responses": []any{"Response A", "Response B"},
				"seed":      42,
			},
		},
		{
			Snapshot: "many_responses_deep_prefix_tree_seed_42",
			Body: map[string]any{
				"messages":  []any{map[string]any{"role": "user", "content": "Pick the best"}},
				"swarm":     map[string]any{"agents": []any{mockAgent}},
				"responses": responses25,
				"seed":      42,
			},
		},
		{
			Snapshot: "mixed_output_modes_seed_88",
			Body: map[string]any{
				"messages": []any{
					map[string]any{"role": "user", "content": "Compare these vacation destinations"},
				},
				"swarm": map[string]any{
					"agents": []any{
						map[string]any{"upstream": "mock", "output_mode": "instruction"},
						map[string]any{"upstream": "mock", "output_mode": "json_schema"},
						map[string]any{"upstream": "mock", "output_mode": "tool_call"},
					},
					"weights": []any{"0.4", "0.3", "0.3"},
				},
				"responses": []any{"Kyoto, Japan", "Reykjavik, Iceland", "Patagonia, Argentina"},
				"seed":      88,
			},
		},
	}

	for _, tc := range cases {
		t.Run(tc.Snapshot+"/unary", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runUnary[VectorCompletionsResponseUnaryVectorCompletion](t, c, "/vector/completions", tc.Body)
			normalized, err := NormalizeVectorCompletionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})

		t.Run(tc.Snapshot+"/streaming", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runStreaming(t, c, "/vector/completions", tc.Body,
				func(acc, chunk *VectorCompletionsResponseStreamingVectorCompletionChunk) {
					acc.Push(chunk)
				},
				VectorCompletionChunkToUnary,
			)
			normalized, err := NormalizeVectorCompletionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})
	}
}
