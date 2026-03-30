package tests

import (
	"path/filepath"
	"testing"

	. "github.com/objective-ai/objectiveai-go/objectiveai"
)

func TestAgentCompletionsHTTP(t *testing.T) {
	c := getTestClient(t)
	snapshotsDir := filepath.Join(assetsDir(), "agent", "completions", "client_tests")

	cases := []httpTestCase{
		{
			Snapshot: "test_basic_mock_agent_seed_42",
			Body: map[string]any{
				"messages": []any{},
				"agent":    map[string]any{"upstream": "mock", "output_mode": "instruction"},
				"seed":     42,
			},
		},
		{
			Snapshot: "test_with_developer_and_user_messages",
			Body: map[string]any{
				"messages": []any{
					map[string]any{"role": "developer", "content": "You are a helpful assistant."},
					map[string]any{"role": "user", "content": "What is 2+2?"},
				},
				"agent": map[string]any{"upstream": "mock", "output_mode": "instruction"},
				"seed":  99,
			},
		},
		{
			Snapshot: "test_json_object_response_format",
			Body: map[string]any{
				"messages":        []any{},
				"agent":           map[string]any{"upstream": "mock", "output_mode": "instruction"},
				"response_format": map[string]any{"type": "json_object"},
				"seed":            42,
			},
		},
	}

	for _, tc := range cases {
		t.Run(tc.Snapshot+"/unary", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runUnary[AgentCompletionsResponseUnaryAgentCompletion](t, c, "/agent/completions", tc.Body)
			normalized, err := NormalizeAgentCompletionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})

		t.Run(tc.Snapshot+"/streaming", func(t *testing.T) {
			expected := loadSnapshot(t, snapshotsDir, tc.Snapshot)
			result := runStreaming(t, c, "/agent/completions", tc.Body,
				func(acc, chunk *AgentCompletionsResponseStreamingAgentCompletionChunk) {
					acc.Push(chunk)
				},
				AgentCompletionChunkToUnary,
			)
			normalized, err := NormalizeAgentCompletionForTests(*result)
			if err != nil {
				t.Fatalf("normalize: %v", err)
			}
			assertRoundedMapEqual(t, tc.Snapshot, toMapJSON(t, normalized), expected)
		})
	}
}
