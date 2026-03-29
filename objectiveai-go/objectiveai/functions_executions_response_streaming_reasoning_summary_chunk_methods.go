package objectiveai

// Push accumulates another ReasoningSummaryChunk into this one.
func (v *FunctionsExecutionsResponseStreamingReasoningSummaryChunk) Push(other *FunctionsExecutionsResponseStreamingReasoningSummaryChunk) {
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

	// error: replace
	v.Error = pushReplace(v.Error, other.Error)

	// usage: delegate
	if v.Usage != nil && other.Usage != nil {
		v.Usage.Push(other.Usage)
	} else if other.Usage != nil {
		v.Usage = other.Usage
	}

	// id, created, object, upstream are immutable
}
