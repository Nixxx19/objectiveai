// Strict roundtrip test harness for Go JSON Schema validation.
//
// THIS FILE MUST NEVER BE MODIFIED.
//
// This harness loads the original JSON schemas from objectiveai-json-schema/
// exactly as they are on disk — no normalization, no massaging, no skip.
// The original schema is treated as the canonical source of truth.
//
// The convertToSchema function uses reflection + struct tags to reconstruct
// JSON schemas from generated Go types. Struct properties are derived from
// reflection (proving lossless encoding), while metadata that Go can't
// express (descriptions, constraints, $ref targets) comes from struct tags
// and the Described/SchemaBody interfaces.
//
// If a test fails, the fix belongs in the conversion logic here or in the
// Go code generator — never in the original schemas.
package objectiveai

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"sort"
	"strings"
	"testing"
)

// ---------------------------------------------------------------------------
// Canonical key ordering (matching Rust builder)
// ---------------------------------------------------------------------------

var keywordOrder = []string{
	"title", "description", "type", "enum", "anyOf", "$ref",
	"properties", "additionalProperties", "items",
	"minItems", "maxItems", "minimum", "maximum",
	"pattern", "format", "default",
}

var keywordRank map[string]int

func init() {
	keywordRank = make(map[string]int, len(keywordOrder))
	for i, kw := range keywordOrder {
		keywordRank[kw] = i
	}
}

// ---------------------------------------------------------------------------
// Schema loading
// ---------------------------------------------------------------------------

func schemaDir() string {
	_, filename, _, _ := runtime.Caller(0)
	return filepath.Join(filepath.Dir(filename), "..", "..", "objectiveai-json-schema")
}

var (
	originalSchemas map[string]map[string]any
	allTitlesSorted []string
)

func init() {
	dir := schemaDir()
	entries, err := os.ReadDir(dir)
	if err != nil {
		panic(fmt.Sprintf("reading schema dir %s: %v", dir, err))
	}
	originalSchemas = make(map[string]map[string]any)
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(dir, entry.Name()))
		if err != nil {
			panic(fmt.Sprintf("reading %s: %v", entry.Name(), err))
		}
		dec := json.NewDecoder(strings.NewReader(string(data)))
		dec.UseNumber()
		var schema map[string]any
		if err := dec.Decode(&schema); err != nil {
			panic(fmt.Sprintf("parsing %s: %v", entry.Name(), err))
		}
		if title, ok := schema["title"].(string); ok {
			originalSchemas[title] = schema
		}
	}
	allTitlesSorted = make([]string, 0, len(originalSchemas))
	for title := range originalSchemas {
		allTitlesSorted = append(allTitlesSorted, title)
	}
	sort.Strings(allTitlesSorted)
}

// ---------------------------------------------------------------------------
// Serialization + comparison
// ---------------------------------------------------------------------------

type orderedMap struct {
	keys   []string
	values map[string]any
}

func (o *orderedMap) MarshalJSON() ([]byte, error) {
	var buf strings.Builder
	buf.WriteByte('{')
	for i, k := range o.keys {
		if i > 0 {
			buf.WriteByte(',')
		}
		keyBytes, _ := json.Marshal(k)
		buf.Write(keyBytes)
		buf.WriteByte(':')
		valBytes, _ := json.Marshal(o.values[k])
		buf.Write(valBytes)
	}
	buf.WriteByte('}')
	return []byte(buf.String()), nil
}

func orderKeys(value any, insideProperties bool) any {
	switch v := value.(type) {
	case map[string]any:
		recursed := make(map[string]any, len(v))
		for k, val := range v {
			recursed[k] = orderKeys(val, k == "properties")
		}
		keys := make([]string, 0, len(recursed))
		for k := range recursed {
			keys = append(keys, k)
		}
		if insideProperties {
			sort.Strings(keys)
		} else {
			unknownRank := len(keywordOrder)
			sort.SliceStable(keys, func(i, j int) bool {
				ri, oki := keywordRank[keys[i]]
				if !oki {
					ri = unknownRank
				}
				rj, okj := keywordRank[keys[j]]
				if !okj {
					rj = unknownRank
				}
				return ri < rj
			})
		}
		return &orderedMap{keys: keys, values: recursed}
	case []any:
		result := make([]any, len(v))
		for i, item := range v {
			result[i] = orderKeys(item, false)
		}
		return result
	default:
		return value
	}
}

func serialize(schema map[string]any) string {
	ordered := orderKeys(schema, false)
	data, _ := json.MarshalIndent(ordered, "", "  ")
	return string(data)
}

func assertSchemaMatches(t *testing.T, title string, converted map[string]any) {
	t.Helper()
	original, ok := originalSchemas[title]
	if !ok {
		t.Fatalf("title %q not found in original schemas", title)
	}
	expected := serialize(original)
	actual := serialize(converted)
	if actual != expected {
		t.Errorf("Schema mismatch for %q:\n\n--- Expected ---\n%s\n\n--- Got ---\n%s",
			title, expected, actual)
	}
}

// ---------------------------------------------------------------------------
// Reflection-based schema converter
// ---------------------------------------------------------------------------

// convertToSchema reconstructs a JSON Schema from a Go type using reflection.
// For struct types: properties are derived from struct fields + tags.
// For non-struct types: the SchemaBody interface provides the body.
func convertToSchema(v any) map[string]any {
	d, ok := v.(Described)
	if !ok {
		return nil
	}

	result := map[string]any{
		"title": d.SchemaTitle(),
	}
	if desc := d.SchemaDescription(); desc != "" {
		result["description"] = desc
	}

	// Check if this is a struct type (properties derived from reflection)
	rv := reflect.ValueOf(v)
	rt := rv.Type()
	if rt.Kind() == reflect.Struct && rt.NumField() > 0 && !isSchemaHelper(rt) {
		// It's a real struct — derive properties from fields
		result["type"] = "object"

		// Check for Body() providing anyOf/$ref (adjacently-tagged structs)
		if sb, ok := v.(SchemaBody); ok {
			body := sb.Body()
			for k, val := range body {
				result[k] = val
			}
		}

		properties := convertStructProperties(v, rt)
		if len(properties) > 0 {
			result["properties"] = properties
		}
	} else {
		// Non-struct type — get body from SchemaBody
		if sb, ok := v.(SchemaBody); ok {
			body := sb.Body()
			for k, val := range body {
				result[k] = val
			}
		}
	}

	return result
}

// isSchemaHelper returns true if the type is a Schema helper struct (empty struct
// used to hold methods for non-struct schema types).
func isSchemaHelper(t reflect.Type) bool {
	return t.NumField() == 0
}

// fieldDescriber is implemented by structs that have per-field descriptions.
type fieldDescriber interface {
	FieldDescriptions() map[string]string
}

// convertStructProperties uses reflection on struct fields + their tags
// to reconstruct the "properties" object of a JSON Schema.
func convertStructProperties(v any, rt reflect.Type) map[string]any {
	properties := map[string]any{}

	// Get field descriptions from method (not tags, because descriptions can contain backticks)
	var fieldDescs map[string]string
	if fd, ok := v.(fieldDescriber); ok {
		fieldDescs = fd.FieldDescriptions()
	}

	for i := 0; i < rt.NumField(); i++ {
		field := rt.Field(i)
		jsonTag := field.Tag.Get("json")
		if jsonTag == "" || jsonTag == "-" {
			continue
		}
		propName := strings.Split(jsonTag, ",")[0]
		isOmitempty := strings.Contains(jsonTag, "omitempty")

		propSchema := convertFieldToSchema(field, isOmitempty)

		// Add description from FieldDescriptions() method
		if desc, ok := fieldDescs[propName]; ok {
			propSchema["description"] = desc
		}

		properties[propName] = propSchema
	}

	return properties
}

// convertFieldToSchema reconstructs the JSON Schema for a single struct field
// from its Go type and struct tags.
func convertFieldToSchema(field reflect.StructField, isOmitempty bool) map[string]any {
	result := map[string]any{}

	// Note: field descriptions are added by convertStructProperties from
	// the FieldDescriptions() method, not from struct tags.

	// Check for $ref and nullable (from tags)
	ref := field.Tag.Get("ref")
	isNullableField := field.Tag.Get("nullable") == "true"

	ft := field.Type

	if isNullableField {
		// Nullable field → anyOf: [nonNullSchema, {type: null}]
		innerType := ft
		if ft.Kind() == reflect.Ptr {
			innerType = ft.Elem()
		}
		nonNullSchema := buildTypeSchema(innerType, field)
		if ref != "" {
			nonNullSchema = map[string]any{"$ref": ref}
		}
		result["anyOf"] = []any{nonNullSchema, map[string]any{"type": "null"}}
	} else if ref != "" {
		// Non-nullable $ref
		result["$ref"] = ref
	} else {
		// Non-nullable, non-ref — inline type schema
		typeSchema := buildTypeSchema(ft, field)
		for k, v := range typeSchema {
			result[k] = v
		}
	}

	// Default value (on the outer schema, not inside anyOf)
	if def := field.Tag.Get("def"); def != "" {
		result["default"] = parseDefault(def)
	}

	return result
}

// buildTypeSchema converts a Go type to its JSON Schema type representation.
func buildTypeSchema(t reflect.Type, field reflect.StructField) map[string]any {
	result := map[string]any{}

	// Enum values from tag
	if enumStr := field.Tag.Get("enum"); enumStr != "" {
		vals := strings.Split(enumStr, ",")
		enumAny := make([]any, len(vals))
		for i, v := range vals {
			enumAny[i] = v
		}
		result["enum"] = enumAny
	}

	switch t.Kind() {
	case reflect.String:
		result["type"] = "string"
		if f := field.Tag.Get("fmt"); f != "" {
			result["format"] = f
		}
		if p := field.Tag.Get("pat"); p != "" {
			result["pattern"] = p
		}
	case reflect.Bool:
		result["type"] = "boolean"
	case reflect.Float32, reflect.Float64:
		result["type"] = "number"
		addNumericConstraints(result, field)
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		result["type"] = "integer"
		addIntConstraintsFromGoType(result, t, field)
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		result["type"] = "integer"
		addIntConstraintsFromGoType(result, t, field)
	case reflect.Slice:
		result["type"] = "array"
		if iref := field.Tag.Get("items_ref"); iref != "" {
			result["items"] = map[string]any{"$ref": iref}
		} else {
			itemType := t.Elem()
			if itemType.Kind() != reflect.Interface {
				items := buildItemTypeSchema(itemType)
				// Add item constraints from tags
				if v := field.Tag.Get("items_min"); v != "" {
					items["minimum"] = json.Number(v)
				}
				if v := field.Tag.Get("items_max"); v != "" {
					items["maximum"] = json.Number(v)
				}
				if v := field.Tag.Get("items_fmt"); v != "" {
					items["format"] = v
				}
				result["items"] = items
			}
		}
	case reflect.Map:
		result["type"] = "object"
		// Check for additionalProperties $ref
		if apRef := field.Tag.Get("addprops_ref"); apRef != "" {
			result["additionalProperties"] = map[string]any{"$ref": apRef}
		} else {
			valType := t.Elem()
			if valType.Kind() != reflect.Interface {
				ap := buildItemTypeSchema(valType)
				if v := field.Tag.Get("addprops_min"); v != "" {
					ap["minimum"] = json.Number(v)
				}
				if v := field.Tag.Get("addprops_max"); v != "" {
					ap["maximum"] = json.Number(v)
				}
				result["additionalProperties"] = ap
			}
		}
		// Check for explicit additionalProperties tag (boolean)
		if apTag := field.Tag.Get("addprops"); apTag == "true" {
			result["additionalProperties"] = true
		}
	case reflect.Interface:
		// any → empty schema (or with addprops tag)
		if ap := field.Tag.Get("addprops"); ap == "true" {
			result["type"] = "object"
			result["additionalProperties"] = true
		}
	default:
		// unknown type → empty schema
	}

	return result
}

// buildItemTypeSchema converts an element type (array items, map values) to JSON Schema.
func buildItemTypeSchema(t reflect.Type) map[string]any {
	result := map[string]any{}
	switch t.Kind() {
	case reflect.String:
		result["type"] = "string"
	case reflect.Bool:
		result["type"] = "boolean"
	case reflect.Float32, reflect.Float64:
		result["type"] = "number"
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		result["type"] = "integer"
		addIntConstraintsFromGoTypeOnly(result, t)
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		result["type"] = "integer"
		addIntConstraintsFromGoTypeOnly(result, t)
	case reflect.Interface:
		// any → empty schema
	default:
		// complex element type
	}
	return result
}

// addNumericConstraints adds minimum/maximum from struct tags.
func addNumericConstraints(result map[string]any, field reflect.StructField) {
	if min := field.Tag.Get("min"); min != "" {
		result["minimum"] = json.Number(min)
	}
	if max := field.Tag.Get("max"); max != "" {
		result["maximum"] = json.Number(max)
	}
}

// addIntConstraintsFromGoType adds minimum/maximum based on Go integer type
// and/or struct tags.
func addIntConstraintsFromGoType(result map[string]any, t reflect.Type, field reflect.StructField) {
	// Prefer explicit tags if present
	if min := field.Tag.Get("min"); min != "" {
		result["minimum"] = json.Number(min)
	} else {
		result["minimum"] = goIntMinimum(t)
	}
	if max := field.Tag.Get("max"); max != "" {
		result["maximum"] = json.Number(max)
	} else {
		result["maximum"] = goIntMaximum(t)
	}
}

// addIntConstraintsFromGoTypeOnly adds min/max from Go type only (no tags).
func addIntConstraintsFromGoTypeOnly(result map[string]any, t reflect.Type) {
	result["minimum"] = goIntMinimum(t)
	result["maximum"] = goIntMaximum(t)
}

func goIntMinimum(t reflect.Type) json.Number {
	switch t.Kind() {
	case reflect.Int8:
		return json.Number(fmt.Sprintf("%d", math.MinInt8))
	case reflect.Int16:
		return json.Number(fmt.Sprintf("%d", math.MinInt16))
	case reflect.Int32:
		return json.Number(fmt.Sprintf("%d", math.MinInt32))
	case reflect.Int64:
		return json.Number(fmt.Sprintf("%d", math.MinInt64))
	case reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return json.Number("0")
	}
	return json.Number("0")
}

func goIntMaximum(t reflect.Type) json.Number {
	switch t.Kind() {
	case reflect.Int8:
		return json.Number(fmt.Sprintf("%d", math.MaxInt8))
	case reflect.Int16:
		return json.Number(fmt.Sprintf("%d", math.MaxInt16))
	case reflect.Int32:
		return json.Number(fmt.Sprintf("%d", math.MaxInt32))
	case reflect.Int64:
		return json.Number(fmt.Sprintf("%d", math.MaxInt64))
	case reflect.Uint8:
		return json.Number(fmt.Sprintf("%d", math.MaxUint8))
	case reflect.Uint16:
		return json.Number(fmt.Sprintf("%d", math.MaxUint16))
	case reflect.Uint32:
		return json.Number(fmt.Sprintf("%d", math.MaxUint32))
	case reflect.Uint64:
		return json.Number("18446744073709551615")
	}
	return json.Number("0")
}

// parseDefault parses a default value string from a struct tag.
func parseDefault(s string) any {
	if s == "null" {
		return nil
	}
	if s == "true" {
		return true
	}
	if s == "false" {
		return false
	}
	// Try as number
	n := json.Number(s)
	if _, err := n.Int64(); err == nil {
		return n
	}
	if _, err := n.Float64(); err == nil {
		return n
	}
	// Return as string
	return s
}
