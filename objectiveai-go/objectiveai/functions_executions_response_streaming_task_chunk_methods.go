package objectiveai

// Push accumulates another TaskChunk into this one.
// Variant dispatch: only merges if both are the same variant type.
func (v *FunctionsExecutionsResponseStreamingTaskChunk) Push(other *FunctionsExecutionsResponseStreamingTaskChunk) {
	if v.Variant1 != nil && other.Variant1 != nil {
		v.Variant1.Push(other.Variant1)
	} else if v.Variant2 != nil && other.Variant2 != nil {
		v.Variant2.Push(other.Variant2)
	}
}
