package objectiveai

// Push accumulates another MessageChunk into this one.
// Only merges if both are assistant variants.
func (v *AgentCompletionsResponseStreamingMessageChunk) Push(other *AgentCompletionsResponseStreamingMessageChunk) {
	if v.Assistant != nil && other.Assistant != nil {
		v.Assistant.Push(other.Assistant)
	}
}
