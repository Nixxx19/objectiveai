package objectiveai

// Push accumulates another MessageChunk into this one.
// Only merges if both are assistant variants (Variant1).
func (v *AgentCompletionsResponseStreamingMessageChunk) Push(other *AgentCompletionsResponseStreamingMessageChunk) {
	if v.Variant1 != nil && other.Variant1 != nil {
		v.Variant1.Push(other.Variant1)
	}
}
