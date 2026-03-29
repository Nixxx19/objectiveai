package objectiveai

// Push accumulates another vector completions AgentCompletionChunk into this one.
func (v *VectorCompletionsResponseStreamingAgentCompletionChunk) Push(other *VectorCompletionsResponseStreamingAgentCompletionChunk) {
	// messages: merge by index
	pushByIndex(&v.Messages, other.Messages,
		func(m *AgentCompletionsResponseStreamingMessageChunk) uint64 {
			if m.Assistant != nil {
				return m.Assistant.Index
			}
			return 0
		},
		func(a, b *AgentCompletionsResponseStreamingMessageChunk) { a.Push(b) },
	)

	// usage: delegate
	if v.Usage != nil && other.Usage != nil {
		v.Usage.Push(other.Usage)
	} else if other.Usage != nil {
		v.Usage = other.Usage
	}

	// error: replace
	v.Error = pushReplace(v.Error, other.Error)

	// id, created, object, upstream, index are immutable
}
