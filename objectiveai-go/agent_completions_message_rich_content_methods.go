package objectiveai

// Push accumulates another RichContent into this one.
// Union dispatch: text+text -> concat, text+parts -> convert, parts+text -> append, parts+parts -> extend.
func (v *AgentCompletionsMessageRichContent) Push(other *AgentCompletionsMessageRichContent) {
	selfIsText := v.Variant1 != nil
	otherIsText := other.Variant1 != nil

	switch {
	case selfIsText && otherIsText:
		// text + text -> concatenate
		s := *v.Variant1 + *other.Variant1
		v.Variant1 = &s

	case selfIsText && !otherIsText:
		// text + parts -> convert self to parts, extend
		textPart := AgentCompletionsMessageRichContentPart{
			Variant1: &AgentCompletionsMessageRichContentPartVariant1{
				Text: *v.Variant1,
				Type: "text",
			},
		}
		parts := make([]AgentCompletionsMessageRichContentPart, 0, 1+len(other.Variant2))
		parts = append(parts, textPart)
		parts = append(parts, other.Variant2...)
		v.Variant1 = nil
		v.Variant2 = parts

	case !selfIsText && otherIsText:
		// parts + text -> append text as new part
		if other.Variant1 != nil && *other.Variant1 != "" {
			textPart := AgentCompletionsMessageRichContentPart{
				Variant1: &AgentCompletionsMessageRichContentPartVariant1{
					Text: *other.Variant1,
					Type: "text",
				},
			}
			v.Variant2 = append(v.Variant2, textPart)
		}

	default:
		// parts + parts -> extend
		if len(other.Variant2) > 0 {
			v.Variant2 = append(v.Variant2, other.Variant2...)
		}
	}
}
