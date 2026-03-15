package objectiveai

// Push accumulates another AgentCompletionChunk into this one.
func (v *AgentCompletionsResponseStreamingAgentCompletionChunk) Push(other *AgentCompletionsResponseStreamingAgentCompletionChunk) {
	// messages: merge by index
	pushByIndex(&v.Messages, other.Messages,
		func(m *AgentCompletionsResponseStreamingMessageChunk) uint64 {
			if m.Variant1 != nil {
				return m.Variant1.Index
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

	// id, created, object, upstream are immutable
}
