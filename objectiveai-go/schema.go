// Package objectiveai provides auto-generated Go types for the ObjectiveAI API.
//
// Types are generated from JSON Schema files in objectiveai-json-schema/
// by scripts/install_go.go. Do not edit generated files directly.
package objectiveai

// SchemaProvider is implemented by every generated type.
// It returns the JSON Schema metadata that cannot be inferred from
// struct tags alone (title, description, top-level structure).
type SchemaProvider interface {
	// JSONSchema returns a map representing the JSON Schema for this type.
	// The map uses the canonical key set: title, description, type, enum,
	// anyOf, $ref, properties, additionalProperties, items, minItems,
	// maxItems, minimum, maximum, pattern, format, default.
	JSONSchema() map[string]any
}
