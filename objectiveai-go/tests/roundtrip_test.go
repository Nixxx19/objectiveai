package tests

import (
	"encoding/json"
	"go/ast"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

// ---------------------------------------------------------------------------
// AST-based type info extraction
// ---------------------------------------------------------------------------

type typeInfo struct {
	name        string
	doc         string
	isAlias     bool
	aliasTarget string
	fields      []fieldInfo
	embeds      []string
	methods     map[string]string
}

type fieldInfo struct {
	name     string
	typeName string
	doc      string
	tags     string
}

func parseSourceDir(t *testing.T) map[string]*typeInfo {
	t.Helper()
	dir := SourceDir()

	fset := token.NewFileSet()
	types := map[string]*typeInfo{}

	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("reading source dir: %v", err)
	}

	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".go") {
			continue
		}
		if strings.HasSuffix(entry.Name(), "_test.go") {
			continue
		}

		f, err := parser.ParseFile(fset, filepath.Join(dir, entry.Name()), nil, parser.ParseComments)
		if err != nil {
			t.Fatalf("parsing %s: %v", entry.Name(), err)
		}

		for _, decl := range f.Decls {
			switch d := decl.(type) {
			case *ast.GenDecl:
				for _, spec := range d.Specs {
					ts, ok := spec.(*ast.TypeSpec)
					if !ok {
						continue
					}
					ti := &typeInfo{
						name:    ts.Name.Name,
						methods: map[string]string{},
					}

					if d.Doc != nil {
						ti.doc = strings.TrimSpace(d.Doc.Text())
					} else if ts.Doc != nil {
						ti.doc = strings.TrimSpace(ts.Doc.Text())
					}

					if ts.Assign.IsValid() {
						ti.isAlias = true
						ti.aliasTarget = typeExprString(ts.Type)
					} else if st, ok := ts.Type.(*ast.StructType); ok {
						for _, field := range st.Fields.List {
							if len(field.Names) == 0 {
								ti.embeds = append(ti.embeds, typeExprString(field.Type))
								continue
							}
							fi := fieldInfo{
								name:     field.Names[0].Name,
								typeName: typeExprString(field.Type),
							}
							if field.Doc != nil {
								fi.doc = strings.TrimSpace(field.Doc.Text())
							}
							if field.Tag != nil {
								fi.tags = strings.TrimPrefix(strings.TrimSuffix(field.Tag.Value, "`"), "`")
							}
							ti.fields = append(ti.fields, fi)
						}
					}

					types[ts.Name.Name] = ti
				}

			case *ast.FuncDecl:
				if d.Recv == nil || len(d.Recv.List) == 0 {
					continue
				}
				recvType := typeExprString(d.Recv.List[0].Type)
				methodName := d.Name.Name

				if d.Body != nil && len(d.Body.List) == 1 {
					if ret, ok := d.Body.List[0].(*ast.ReturnStmt); ok && len(ret.Results) == 1 {
						if lit, ok := ret.Results[0].(*ast.BasicLit); ok && lit.Kind == token.STRING {
							val := strings.Trim(lit.Value, "\"")
							if ti, ok := types[recvType]; ok {
								ti.methods[methodName] = val
							} else {
								ti := &typeInfo{name: recvType, methods: map[string]string{}}
								ti.methods[methodName] = val
								types[recvType] = ti
							}
						}
					}
				}
			}
		}
	}

	return types
}

func typeExprString(expr ast.Expr) string {
	switch e := expr.(type) {
	case *ast.Ident:
		return e.Name
	case *ast.StarExpr:
		return "*" + typeExprString(e.X)
	case *ast.ArrayType:
		return "[]" + typeExprString(e.Elt)
	case *ast.MapType:
		return "map[" + typeExprString(e.Key) + "]" + typeExprString(e.Value)
	case *ast.SelectorExpr:
		return typeExprString(e.X) + "." + e.Sel.Name
	case *ast.InterfaceType:
		return "any"
	default:
		return "any"
	}
}

// ---------------------------------------------------------------------------
// Schema reconstruction
// ---------------------------------------------------------------------------

func buildTitleMap(types map[string]*typeInfo) map[string]string {
	m := map[string]string{}
	for goName, ti := range types {
		if title, ok := ti.methods["SchemaTitle"]; ok {
			m[goName] = title
		}
	}
	return m
}

func buildReverseTitleMap(titleMap map[string]string) map[string]string {
	m := map[string]string{}
	for goName, title := range titleMap {
		m[title] = goName
	}
	return m
}

func getTagValue(tags string, key string) string {
	return reflect.StructTag(tags).Get(key)
}

func reconstructSchema(
	goName string,
	types map[string]*typeInfo,
	titleMap map[string]string,
) map[string]any {
	ti, ok := types[goName]
	if !ok {
		return nil
	}

	title := titleMap[goName]
	result := map[string]any{"title": title}
	if ti.doc != "" {
		result["description"] = ti.doc
	}

	if ti.isAlias {
		target := ti.aliasTarget
		if strings.HasPrefix(target, "*") {
			inner := strings.TrimPrefix(target, "*")
			if innerTitle, ok := titleMap[inner]; ok {
				result["anyOf"] = []any{
					map[string]any{"$ref": innerTitle},
					map[string]any{"type": "null"},
				}
			}
		}
		return result
	}

	// Variant struct: has MarshalJSON and fields without json tags
	_, hasMarshal := ti.methods["MarshalJSON"]
	isVariant := hasMarshal && len(ti.fields) > 0 && getTagValue(ti.fields[0].tags, "json") == ""

	if isVariant {
		return reconstructVariantSchema(ti, types, titleMap, result)
	}

	return reconstructStructSchema(ti, types, titleMap, result)
}

func reconstructStructSchema(
	ti *typeInfo,
	types map[string]*typeInfo,
	titleMap map[string]string,
	result map[string]any,
) map[string]any {
	result["type"] = "object"

	for _, embed := range ti.embeds {
		if embedTitle, ok := titleMap[embed]; ok {
			result["$ref"] = embedTitle
		}
	}

	properties := map[string]any{}
	for _, f := range ti.fields {
		jsonTag := getTagValue(f.tags, "json")
		if jsonTag == "" || jsonTag == "-" {
			continue
		}
		propName := strings.Split(jsonTag, ",")[0]
		isOmitempty := strings.Contains(jsonTag, "omitempty")

		propSchema := reconstructFieldSchema(f, isOmitempty, types, titleMap)
		if f.doc != "" {
			propSchema["description"] = f.doc
		}
		properties[propName] = propSchema
	}

	if len(properties) > 0 {
		result["properties"] = properties
	}

	return result
}

func reconstructFieldSchema(
	f fieldInfo,
	isOmitempty bool,
	types map[string]*typeInfo,
	titleMap map[string]string,
) map[string]any {
	validateTag := getTagValue(f.tags, "validate")
	patternTag := getTagValue(f.tags, "pattern")
	defaultTag := getTagValue(f.tags, "default")

	typeName := f.typeName
	isPointer := strings.HasPrefix(typeName, "*")
	if isPointer {
		typeName = strings.TrimPrefix(typeName, "*")
	}

	isNullable := isPointer && isOmitempty

	inner := buildFieldTypeSchema(typeName, types, titleMap)

	if patternTag != "" {
		inner["pattern"] = patternTag
	}
	if defaultTag != "" {
		inner["default"] = parseDefaultValue(defaultTag)
	}
	addValidateConstraints(inner, validateTag)

	if isNullable {
		return map[string]any{
			"anyOf": []any{inner, map[string]any{"type": "null"}},
		}
	}
	return inner
}

func buildFieldTypeSchema(
	typeName string,
	types map[string]*typeInfo,
	titleMap map[string]string,
) map[string]any {
	if schemaTitle, ok := titleMap[typeName]; ok {
		return map[string]any{"$ref": schemaTitle}
	}

	switch typeName {
	case "string":
		return map[string]any{"type": "string"}
	case "bool":
		return map[string]any{"type": "boolean"}
	case "float64":
		return map[string]any{"type": "number"}
	case "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64":
		result := map[string]any{"type": "integer"}
		addIntConstraints(result, typeName)
		return result
	case "any":
		return map[string]any{}
	case "time.Time":
		return map[string]any{"type": "string", "format": "date-time"}
	case "uuid.UUID":
		return map[string]any{"type": "string", "format": "uuid"}
	}

	if strings.HasPrefix(typeName, "[]") {
		elemType := strings.TrimPrefix(typeName, "[]")
		items := buildFieldTypeSchema(elemType, types, titleMap)
		return map[string]any{"type": "array", "items": items}
	}

	if strings.HasPrefix(typeName, "map[string]") {
		valType := strings.TrimPrefix(typeName, "map[string]")
		if valType == "any" {
			return map[string]any{"type": "object", "additionalProperties": true}
		}
		valSchema := buildFieldTypeSchema(valType, types, titleMap)
		return map[string]any{"type": "object", "additionalProperties": valSchema}
	}

	return map[string]any{}
}

func addValidateConstraints(schema map[string]any, validateTag string) {
	if validateTag == "" {
		return
	}
	for _, part := range strings.Split(validateTag, ",") {
		if strings.HasPrefix(part, "oneof=") {
			vals := strings.Split(strings.TrimPrefix(part, "oneof="), " ")
			enumVals := make([]any, len(vals))
			for i, v := range vals {
				enumVals[i] = v
			}
			schema["enum"] = enumVals
		}
		if strings.HasPrefix(part, "min=") {
			schema["minimum"] = json.Number(strings.TrimPrefix(part, "min="))
		}
		if strings.HasPrefix(part, "max=") {
			schema["maximum"] = json.Number(strings.TrimPrefix(part, "max="))
		}
	}
}

func addIntConstraints(result map[string]any, typeName string) {
	switch typeName {
	case "int8":
		result["minimum"] = json.Number("-128")
		result["maximum"] = json.Number("127")
	case "int16":
		result["minimum"] = json.Number("-32768")
		result["maximum"] = json.Number("32767")
	case "int32":
		result["minimum"] = json.Number("-2147483648")
		result["maximum"] = json.Number("2147483647")
	case "uint8":
		result["minimum"] = json.Number("0")
		result["maximum"] = json.Number("255")
	case "uint16":
		result["minimum"] = json.Number("0")
		result["maximum"] = json.Number("65535")
	case "uint32":
		result["minimum"] = json.Number("0")
		result["maximum"] = json.Number("4294967295")
	case "uint64":
		result["minimum"] = json.Number("0")
		result["maximum"] = json.Number("18446744073709551615")
	}
}

func parseDefaultValue(s string) any {
	if s == "true" {
		return true
	}
	if s == "false" {
		return false
	}
	if s == "null" {
		return nil
	}
	var n json.Number
	if err := json.Unmarshal([]byte(s), &n); err == nil {
		return n
	}
	return s
}

func reconstructVariantSchema(
	ti *typeInfo,
	types map[string]*typeInfo,
	titleMap map[string]string,
	result map[string]any,
) map[string]any {
	var anyOf []any
	for _, f := range ti.fields {
		variant := reconstructVariant(f, types, titleMap)
		anyOf = append(anyOf, variant)
	}
	result["anyOf"] = anyOf
	return result
}

func reconstructVariant(
	f fieldInfo,
	types map[string]*typeInfo,
	titleMap map[string]string,
) map[string]any {
	variant := map[string]any{"title": f.name}

	typeName := f.typeName
	if strings.HasPrefix(typeName, "*") {
		typeName = strings.TrimPrefix(typeName, "*")
	}

	if subTi, ok := types[typeName]; ok && !subTi.isAlias {
		// Embedded type → adjacently-tagged ($ref + properties)
		if len(subTi.embeds) > 0 {
			embedType := subTi.embeds[0]
			if embedTitle, ok := titleMap[embedType]; ok {
				variant["$ref"] = embedTitle
			}
			variant["type"] = "object"
			props := map[string]any{}
			for _, sf := range subTi.fields {
				jsonTag := getTagValue(sf.tags, "json")
				if jsonTag == "" {
					continue
				}
				propName := strings.Split(jsonTag, ",")[0]
				propSchema := reconstructFieldSchema(sf, strings.Contains(jsonTag, "omitempty"), types, titleMap)
				props[propName] = propSchema
			}
			if len(props) > 0 {
				variant["properties"] = props
			}
			return variant
		}

		// Sub-struct with SchemaTitle → $ref
		if subTitle, ok := titleMap[typeName]; ok {
			variant["$ref"] = subTitle
			return variant
		}

		// Inline object without embedding
		variant["type"] = "object"
		props := map[string]any{}
		for _, sf := range subTi.fields {
			jsonTag := getTagValue(sf.tags, "json")
			if jsonTag == "" {
				continue
			}
			propName := strings.Split(jsonTag, ",")[0]
			propSchema := reconstructFieldSchema(sf, strings.Contains(jsonTag, "omitempty"), types, titleMap)
			props[propName] = propSchema
		}
		if len(props) > 0 {
			variant["properties"] = props
		}
		return variant
	}

	// Known schema type → $ref
	if refTitle, ok := titleMap[typeName]; ok {
		variant["$ref"] = refTitle
		return variant
	}

	// Primitive variant (string with enum)
	validateTag := getTagValue(f.tags, "validate")
	inner := buildFieldTypeSchema(typeName, types, titleMap)
	addValidateConstraints(inner, validateTag)
	for k, v := range inner {
		variant[k] = v
	}
	return variant
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

func TestRoundtrip(t *testing.T) {
	types := parseSourceDir(t)
	titleMap := buildTitleMap(types)
	reverseMap := buildReverseTitleMap(titleMap)

	for _, title := range AllTitlesSorted {
		goName, ok := reverseMap[title]
		if !ok {
			t.Run(title, func(t *testing.T) {
				t.Fatalf("no Go type with SchemaTitle() = %q", title)
			})
			continue
		}

		t.Run(title, func(t *testing.T) {
			schema := reconstructSchema(goName, types, titleMap)
			if schema == nil {
				t.Fatalf("failed to reconstruct schema for %s", goName)
			}
			AssertSchemaMatches(t, title, schema)
		})
	}
}
