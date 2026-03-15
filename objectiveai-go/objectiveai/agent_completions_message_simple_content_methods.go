package objectiveai

// Push accumulates another SimpleContent into this one.
// Union dispatch: text+text -> concat, text+parts -> convert, parts+text -> append, parts+parts -> extend.
func (v *AgentCompletionsMessageSimpleContent) Push(other *AgentCompletionsMessageSimpleContent) {
	selfIsText := v.Variant1 != nil
	otherIsText := other.Variant1 != nil

	switch {
	case selfIsText && otherIsText:
		s := *v.Variant1 + *other.Variant1
		v.Variant1 = &s

	case selfIsText && !otherIsText:
		textPart := AgentCompletionsMessageSimpleContentPart{
			Text: *v.Variant1,
			Type: "text",
		}
		parts := make([]AgentCompletionsMessageSimpleContentPart, 0, 1+len(other.Variant2))
		parts = append(parts, textPart)
		parts = append(parts, other.Variant2...)
		v.Variant1 = nil
		v.Variant2 = parts

	case !selfIsText && otherIsText:
		if other.Variant1 != nil && *other.Variant1 != "" {
			textPart := AgentCompletionsMessageSimpleContentPart{
				Text: *other.Variant1,
				Type: "text",
			}
			v.Variant2 = append(v.Variant2, textPart)
		}

	default:
		if len(other.Variant2) > 0 {
			v.Variant2 = append(v.Variant2, other.Variant2...)
		}
	}
}
