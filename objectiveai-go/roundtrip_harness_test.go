// Strict roundtrip test harness for Go JSON Schema validation.
//
// THIS FILE MUST NEVER BE MODIFIED.
//
// This harness is purposefully strict. It loads the original JSON schemas from
// objectiveai-json-schema/ exactly as they are on disk — no normalization, no
// massaging, no skip. The original schema is treated as the canonical source
// of truth and is never altered.
//
// The contract is simple: the caller passes a schema title and a map. This
// harness loads the original, serializes both sides using the canonical key
// ordering from the JSON schema builder (objectiveai-json-schema/builder/),
// and compares the serialized strings for exact equality.
//
// Key ordering rules (matching the Rust builder):
//   - Inside "properties": keys are sorted alphabetically.
//   - Outside "properties": keys are sorted by KEYWORD_ORDER, with any
//     unknown keys placed at the end.
//
// If a test fails, the fix belongs in the caller's conversion/normalization
// logic or in the Go code generator — never in this file.
package objectiveai

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"
)

// keywordOrder is the canonical key ordering for JSON Schema keywords.
// Matches KEYWORD_ORDER in objectiveai-json-schema/builder/src/main.rs.
var keywordOrder = []string{
	"title",
	"description",
	"type",
	"enum",
	"anyOf",
	"$ref",
	"properties",
	"additionalProperties",
	"items",
	"minItems",
	"maxItems",
	"minimum",
	"maximum",
	"pattern",
	"format",
	"default",
}

var keywordRank map[string]int

func init() {
	keywordRank = make(map[string]int, len(keywordOrder))
	for i, kw := range keywordOrder {
		keywordRank[kw] = i
	}
}

// schemaDir returns the path to objectiveai-json-schema/ relative to this file.
func schemaDir() string {
	_, filename, _, _ := runtime.Caller(0)
	return filepath.Join(filepath.Dir(filename), "..", "objectiveai-json-schema")
}

// loadOriginalJSONSchemas loads all JSON schemas from objectiveai-json-schema/
// exactly as-is. Returns a map from each schema's "title" to its raw parsed
// content. Uses json.Number for numeric precision.
func loadOriginalJSONSchemas() (map[string]map[string]any, error) {
	dir := schemaDir()
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("reading schema dir %s: %w", dir, err)
	}

	schemas := make(map[string]map[string]any)
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}
		data, err := os.ReadFile(filepath.Join(dir, entry.Name()))
		if err != nil {
			return nil, fmt.Errorf("reading %s: %w", entry.Name(), err)
		}
		dec := json.NewDecoder(strings.NewReader(string(data)))
		dec.UseNumber()
		var schema map[string]any
		if err := dec.Decode(&schema); err != nil {
			return nil, fmt.Errorf("parsing %s: %w", entry.Name(), err)
		}
		if title, ok := schema["title"].(string); ok {
			schemas[title] = schema
		}
	}
	return schemas, nil
}

// originalSchemas and allTitles are loaded once at init time.
var (
	originalSchemas map[string]map[string]any
	allTitles       map[string]struct{}
	allTitlesSorted []string
)

func init() {
	var err error
	originalSchemas, err = loadOriginalJSONSchemas()
	if err != nil {
		panic(fmt.Sprintf("failed to load original JSON schemas: %v", err))
	}
	allTitles = make(map[string]struct{}, len(originalSchemas))
	allTitlesSorted = make([]string, 0, len(originalSchemas))
	for title := range originalSchemas {
		allTitles[title] = struct{}{}
		allTitlesSorted = append(allTitlesSorted, title)
	}
	sort.Strings(allTitlesSorted)
}

// orderKeys recursively reorders keys to match the Rust builder's canonical
// order. Inside "properties": keys (field names) are sorted alphabetically.
// Outside "properties": keys are sorted by keywordOrder, with unknown keys
// placed at the end.
func orderKeys(value any, insideProperties bool) any {
	switch v := value.(type) {
	case map[string]any:
		// Recurse first
		recursed := make(map[string]any, len(v))
		for k, val := range v {
			recursed[k] = orderKeys(val, k == "properties")
		}
		// Collect keys
		keys := make([]string, 0, len(recursed))
		for k := range recursed {
			keys = append(keys, k)
		}
		// Sort keys
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
		// Build ordered map (use a JSON-serializable ordered structure)
		ordered := &orderedMap{keys: keys, values: recursed}
		return ordered
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

// orderedMap preserves key order for JSON serialization.
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
		keyBytes, err := json.Marshal(k)
		if err != nil {
			return nil, err
		}
		buf.Write(keyBytes)
		buf.WriteByte(':')
		valBytes, err := json.Marshal(o.values[k])
		if err != nil {
			return nil, err
		}
		buf.Write(valBytes)
	}
	buf.WriteByte('}')
	return []byte(buf.String()), nil
}

// serialize converts a schema dict to a canonical JSON string.
// Applies the builder's key ordering, then pretty-prints with 2-space indent.
func serialize(schema map[string]any) (string, error) {
	ordered := orderKeys(schema, false)
	data, err := json.MarshalIndent(ordered, "", "  ")
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// assertSchemaMatches asserts that a converted schema exactly matches the
// original on disk. Both the original and converted are serialized using the
// canonical key ordering before comparison.
func assertSchemaMatches(t *testing.T, title string, converted map[string]any) {
	t.Helper()

	original, ok := originalSchemas[title]
	if !ok {
		t.Fatalf("title %q not found in original schemas", title)
	}

	expectedStr, err := serialize(original)
	if err != nil {
		t.Fatalf("serializing original for %q: %v", title, err)
	}

	actualStr, err := serialize(converted)
	if err != nil {
		t.Fatalf("serializing converted for %q: %v", title, err)
	}

	if actualStr != expectedStr {
		t.Errorf("Schema mismatch for %q:\n\n--- Expected (original from objectiveai-json-schema/) ---\n%s\n\n--- Got (Go-derived) ---\n%s",
			title, expectedStr, actualStr)
	}
}
