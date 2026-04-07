/*
 * Roundtrip test: C# generated types → JSON Schema must exactly match the
 * original objectiveai-json-schema/ files, proving no information is lost
 * during the code generation.
 *
 * RULES FOR THIS FILE
 * ===================
 *
 * 1. This test code is FORBIDDEN from reading or deserializing the original
 *    JSON schema files. Doing so would amount to cheating — the whole point
 *    is that schemas must be reconstructible entirely from the generated
 *    C# types.
 *
 * 2. The only things imported from the harness are:
 *    - AllTitles: the set of schema title strings (metadata, not content)
 *    - AssertSchemaMatches(title, dict): the strict equality check
 *
 * 3. This test MUST be entirely generic. It must not contain any
 *    schema-specific logic, hardcoded titles, special cases, or
 *    conditional branches for particular types.
 */

using System.ComponentModel;
using System.ComponentModel.DataAnnotations;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;
using ObjectiveAI.Attributes;

namespace ObjectiveAI.Tests;

public class RoundtripTest
{
    public static IEnumerable<object[]> AllTitles =>
        RoundtripHarness.AllTitles.OrderBy(t => t).Select(t => new object[] { t });

    // Map from [JsonSchemaTitle] → C# Type
    private static readonly Dictionary<string, Type> TitleToType = BuildTitleMap();

    // Reverse map from Type → title
    private static readonly Dictionary<Type, string> TypeToTitle = TitleToType.ToDictionary(kv => kv.Value, kv => kv.Key);

    private static Dictionary<string, Type> BuildTitleMap()
    {
        var map = new Dictionary<string, Type>();
        var assembly = typeof(JsonSchemaTitleAttribute).Assembly;
        foreach (var type in assembly.GetExportedTypes())
        {
            var attr = type.GetCustomAttribute<JsonSchemaTitleAttribute>();
            if (attr != null)
                map[attr.Title] = type;
        }
        return map;
    }

    [Theory]
    [MemberData(nameof(AllTitles))]
    public void Roundtrip(string title)
    {
        if (!TitleToType.TryGetValue(title, out var type))
        {
            Assert.Fail($"No C# type found for schema title '{title}'");
            return;
        }

        var converted = ConvertTopLevel(type, title);
        RoundtripHarness.AssertSchemaMatches(title, converted);
    }

    // -----------------------------------------------------------------------
    // Top-level conversion
    // -----------------------------------------------------------------------

    private Dictionary<string, object?> ConvertTopLevel(Type type, string title)
    {
        var result = new Dictionary<string, object?> { ["title"] = title };

        var desc = type.GetCustomAttribute<DescriptionAttribute>()?.Description;
        if (desc != null)
            result["description"] = desc;

        if (type.IsEnum)
        {
            ConvertTopLevelEnum(type, result);
        }
        else
        {
            ConvertTopLevelClass(type, result);
        }

        return result;
    }

    // -----------------------------------------------------------------------
    // Flat enum (type: "string" + enum, no anyOf)
    // -----------------------------------------------------------------------

    private void ConvertTopLevelEnum(Type type, Dictionary<string, object?> result)
    {
        var members = type.GetFields(BindingFlags.Public | BindingFlags.Static);
        bool hasDescriptions = members.Any(m => m.GetCustomAttribute<DescriptionAttribute>() != null);

        if (hasDescriptions)
        {
            var anyOf = new List<object?>();
            foreach (var member in members)
            {
                var variant = new Dictionary<string, object?>();
                var memberDesc = member.GetCustomAttribute<DescriptionAttribute>()?.Description;
                if (memberDesc != null)
                    variant["description"] = memberDesc;
                variant["type"] = "string";
                var jsonName = member.GetCustomAttribute<JsonStringEnumMemberNameAttribute>()?.Name ?? member.Name;
                variant["enum"] = new List<object?> { jsonName };
                anyOf.Add(variant);
            }
            result["anyOf"] = anyOf;
        }
        else
        {
            result["type"] = "string";
            var values = new List<object?>();
            foreach (var member in members)
            {
                var jsonName = member.GetCustomAttribute<JsonStringEnumMemberNameAttribute>()?.Name ?? member.Name;
                values.Add(jsonName);
            }
            result["enum"] = values;
        }
    }

    // -----------------------------------------------------------------------
    // Class conversion (union classes, regular objects, wrappers, etc.)
    // -----------------------------------------------------------------------

    private void ConvertTopLevelClass(Type type, Dictionary<string, object?> result)
    {
        var allProps = type.GetProperties(BindingFlags.Public | BindingFlags.Instance);
        var variantProps = allProps.Where(p => p.GetCustomAttribute<JsonSchemaVariantAttribute>() != null).ToList();
        var regularProps = allProps.Where(p => p.GetCustomAttribute<JsonPropertyNameAttribute>() != null).ToList();
        var refAttr = type.GetCustomAttribute<JsonSchemaRefAttribute>();
        var nullableAttr = allProps.FirstOrDefault(p => p.Name == "Value" && p.GetCustomAttribute<JsonSchemaNullableAttribute>() != null);

        // Value wrapper (single Value property, no [JsonPropertyName])
        if (IsValueWrapper(type, allProps))
        {
            ConvertValueWrapper(type, allProps, result);
            return;
        }

        // Array wrapper (single Items property of List<>)
        if (IsArrayWrapper(type, allProps))
        {
            ConvertArrayWrapper(type, allProps, result);
            return;
        }

        // Nullable wrapper (single Value? property with [JsonSchemaNullable])
        if (nullableAttr != null && allProps.Length == 1)
        {
            ConvertNullableWrapper(type, nullableAttr, result);
            return;
        }

        // Union class (has variant properties)
        if (variantProps.Count > 0)
        {
            var anyOf = variantProps.Select(ConvertVariantProperty).ToList<object?>();
            result["anyOf"] = anyOf;

            // If also has regular properties, this is a flattened model with anyOf
            if (regularProps.Count > 0)
            {
                result["type"] = "object";
                result["properties"] = ConvertProperties(regularProps);
            }
            return;
        }

        // Flattened ref model ($ref + properties)
        if (refAttr != null)
        {
            result["type"] = "object";
            result["$ref"] = refAttr.RefTitle;
            if (regularProps.Count > 0)
                result["properties"] = ConvertProperties(regularProps);
            return;
        }

        // Regular object class
        if (regularProps.Count > 0 || allProps.Length == 0)
        {
            result["type"] = "object";
            if (regularProps.Count > 0)
                result["properties"] = ConvertProperties(regularProps);

            var apAttr = type.GetCustomAttribute<JsonSchemaAdditionalPropertiesAttribute>();
            if (apAttr != null)
                result["additionalProperties"] = apAttr.Allowed;
            return;
        }

        // Empty object
        result["type"] = "object";
    }

    // -----------------------------------------------------------------------
    // Variant property → anyOf entry reconstruction
    // -----------------------------------------------------------------------

    private Dictionary<string, object?> ConvertVariantProperty(PropertyInfo prop)
    {
        var attr = prop.GetCustomAttribute<JsonSchemaVariantAttribute>()!;
        var result = new Dictionary<string, object?>();

        // Title
        result["title"] = attr.Title;

        // Description
        var desc = prop.GetCustomAttribute<DescriptionAttribute>()?.Description;
        if (desc != null)
            result["description"] = desc;

        // Type
        if (attr.Type != null)
            result["type"] = attr.Type;

        // Enum
        if (attr.Enum != null)
            result["enum"] = attr.Enum.Select(v => (object?)v).ToList();

        // $ref
        if (attr.Ref != null)
            result["$ref"] = attr.Ref;

        // For wrapper/inline-object variants, extract properties from the property's type
        var propType = Nullable.GetUnderlyingType(prop.PropertyType) ?? prop.PropertyType;
        var wrapperAttr = propType.GetCustomAttribute<JsonSchemaVariantWrapperAttribute>();

        if (wrapperAttr != null)
        {
            // Discriminated variant wrapper — get properties from the wrapper class
            var wrapperRegularProps = propType.GetProperties(BindingFlags.Public | BindingFlags.Instance | BindingFlags.DeclaredOnly)
                .Where(p => p.GetCustomAttribute<JsonPropertyNameAttribute>() != null).ToList();
            if (wrapperRegularProps.Count > 0)
                result["properties"] = ConvertProperties(wrapperRegularProps);
        }
        else if (attr.Type == "object" && attr.Ref == null && !IsDictionaryType(propType))
        {
            // Inline object variant — get properties from the property's type
            var inlineProps = propType.GetProperties(BindingFlags.Public | BindingFlags.Instance)
                .Where(p => p.GetCustomAttribute<JsonPropertyNameAttribute>() != null).ToList();
            if (inlineProps.Count > 0)
                result["properties"] = ConvertProperties(inlineProps);

            // additionalProperties on inline class
            var apAttr = propType.GetCustomAttribute<JsonSchemaAdditionalPropertiesAttribute>();
            if (apAttr != null)
                result["additionalProperties"] = apAttr.Allowed;

            var apSchemaAttr = propType.GetCustomAttribute<JsonSchemaAdditionalPropertiesSchemaAttribute>();
            if (apSchemaAttr != null)
                AddAdditionalPropertiesSchema(result, apSchemaAttr);
        }
        else if (attr.Type == "object" && attr.Ref == null && IsDictionaryType(propType))
        {
            // Dictionary variant — additionalProperties from the value type
            var apSchemaAttr = prop.GetCustomAttribute<JsonSchemaAdditionalPropertiesSchemaAttribute>();
            if (apSchemaAttr != null)
                AddAdditionalPropertiesSchema(result, apSchemaAttr);
            else
            {
                // Infer additionalProperties from Dictionary<string, V> generic type
                var valType = propType.GetGenericArguments()[1];
                if (valType != typeof(JsonElement))
                    result["additionalProperties"] = TryConvertAsRef(valType) ?? ConvertType(valType);
            }
        }
        else if (attr.Type == "array")
        {
            // Check for stored items schema (complex items with inline anyOf)
            var itemsSchemaAttr = prop.GetCustomAttribute<JsonSchemaItemsSchemaAttribute>();
            if (itemsSchemaAttr != null)
            {
                var itemsEl = JsonDocument.Parse(itemsSchemaAttr.Json).RootElement;
                result["items"] = JsonElementToStructure(itemsEl);
            }
            else
            {
                // Array variant — items from List<T>
                var listType = Nullable.GetUnderlyingType(prop.PropertyType) ?? prop.PropertyType;
                if (listType.IsGenericType && listType.GetGenericTypeDefinition() == typeof(List<>))
                {
                    var itemType = listType.GetGenericArguments()[0];
                    var itemsNullable = prop.GetCustomAttribute<JsonSchemaItemsNullableAttribute>() != null;
                    var itemsRange = prop.GetCustomAttribute<JsonSchemaItemsRangeAttribute>();

                    if (itemsNullable)
                    {
                        var actualItemType = Nullable.GetUnderlyingType(itemType) ?? itemType;
                        var nonNullSchema = TryConvertAsRef(actualItemType) ?? ConvertType(actualItemType);
                        if (itemsRange != null)
                        {
                            if (itemsRange.Minimum != null)
                                nonNullSchema["minimum"] = ParseJsonNumber(itemsRange.Minimum);
                            if (itemsRange.Maximum != null)
                                nonNullSchema["maximum"] = ParseJsonNumber(itemsRange.Maximum);
                        }
                        result["items"] = new Dictionary<string, object?>
                        {
                            ["anyOf"] = new List<object?> { nonNullSchema, new Dictionary<string, object?> { ["type"] = "null" } }
                        };
                    }
                    else
                    {
                        var itemSchema = TryConvertAsRef(itemType) ?? ConvertType(itemType);
                        // Drill down through nested arrays for item constraints
                        var target = itemSchema;
                        while (target.TryGetValue("type", out var iType) && iType as string == "array"
                               && target.TryGetValue("items", out var innerItems) && innerItems is Dictionary<string, object?> innerDict)
                            target = innerDict;

                        if (itemsRange != null)
                        {
                            if (itemsRange.Minimum != null)
                                target["minimum"] = ParseJsonNumber(itemsRange.Minimum);
                            if (itemsRange.Maximum != null)
                                target["maximum"] = ParseJsonNumber(itemsRange.Maximum);
                        }
                        result["items"] = itemSchema;
                    }
                }
            }
        }

        // Range constraints (minimum/maximum)
        var range = prop.GetCustomAttribute<JsonSchemaRangeAttribute>();
        if (range != null)
        {
            if (attr.Type == "array")
            {
                // On array variants, range = minItems/maxItems
                if (range.Minimum != null)
                    result["minItems"] = ParseJsonNumber(range.Minimum);
                if (range.Maximum != null)
                    result["maxItems"] = ParseJsonNumber(range.Maximum);
            }
            else
            {
                if (range.Minimum != null)
                    result["minimum"] = ParseJsonNumber(range.Minimum);
                if (range.Maximum != null)
                    result["maximum"] = ParseJsonNumber(range.Maximum);
            }
        }

        // Pattern
        var regex = prop.GetCustomAttribute<RegularExpressionAttribute>();
        if (regex != null)
            result["pattern"] = regex.Pattern;

        // Format
        var fmt = prop.GetCustomAttribute<JsonSchemaFormatAttribute>();
        if (fmt != null)
            result["format"] = fmt.Format;

        return result;
    }

    // -----------------------------------------------------------------------
    // Property conversion
    // -----------------------------------------------------------------------

    private Dictionary<string, object?> ConvertProperties(List<PropertyInfo> props)
    {
        var result = new Dictionary<string, object?>();
        foreach (var prop in props)
        {
            var jsonName = prop.GetCustomAttribute<JsonPropertyNameAttribute>()?.Name;
            if (jsonName == null) continue;
            result[jsonName] = ConvertProperty(prop);
        }
        return result;
    }

    private Dictionary<string, object?> ConvertProperty(PropertyInfo prop)
    {
        var result = new Dictionary<string, object?>();

        var desc = prop.GetCustomAttribute<DescriptionAttribute>()?.Description;
        if (desc != null)
            result["description"] = desc;

        var propType = prop.PropertyType;
        var isNullable = prop.GetCustomAttribute<JsonSchemaNullableAttribute>() != null;
        var innerType = isNullable
            ? Nullable.GetUnderlyingType(propType) ?? propType
            : propType;

        // Check for stored inline anyOf
        var inlineAnyOf = prop.GetCustomAttribute<JsonSchemaPropertyAnyOfAttribute>();
        if (inlineAnyOf != null && !isNullable)
        {
            // Non-nullable: stored value is the anyOf JSON array
            var anyOfJson = JsonDocument.Parse(inlineAnyOf.Json).RootElement;
            var anyOf = new List<object?>();
            foreach (var variant in anyOfJson.EnumerateArray())
                anyOf.Add(JsonElementToStructure(variant));
            result["anyOf"] = anyOf;
        }
        else if (!isNullable)
        {
            var typeSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType, prop);
            result.MergeFrom(typeSchema);
            AddConstraints(result, prop);
            AddArrayItemConstraints(result, prop);
        }
        else
        {
            // Check if the non-null variant has stored inline anyOf
            var nullableInlineAnyOf = prop.GetCustomAttribute<JsonSchemaPropertyAnyOfAttribute>();
            Dictionary<string, object?> nonNullSchema;
            if (nullableInlineAnyOf != null)
            {
                // Use stored JSON for the non-null variant
                var el = JsonDocument.Parse(nullableInlineAnyOf.Json).RootElement;
                nonNullSchema = (Dictionary<string, object?>)JsonElementToStructure(el)!;
            }
            else
            {
                nonNullSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType, prop);
                AddConstraints(nonNullSchema, prop);
                AddArrayItemConstraints(nonNullSchema, prop);
            }
            result["anyOf"] = new List<object?>
            {
                nonNullSchema,
                new Dictionary<string, object?> { ["type"] = "null" }
            };
        }

        // Default value
        var defaultAttr = prop.GetCustomAttribute<JsonSchemaDefaultAttribute>();
        if (defaultAttr != null)
            result["default"] = ParseJsonDefault(defaultAttr.JsonValue);

        // omitempty
        var omitEmpty = prop.GetCustomAttribute<JsonSchemaOmitEmptyAttribute>();
        if (omitEmpty != null)
            result["omitempty"] = true;

        return result;
    }

    // -----------------------------------------------------------------------
    // Type conversion
    // -----------------------------------------------------------------------

    private Dictionary<string, object?>? TryConvertAsRef(Type type)
    {
        if (TypeToTitle.TryGetValue(type, out var title))
            return new Dictionary<string, object?> { ["$ref"] = title };
        return null;
    }

    private Dictionary<string, object?> ConvertType(Type type, PropertyInfo? prop = null)
    {
        if (type == typeof(string))
        {
            var result = new Dictionary<string, object?> { ["type"] = "string" };
            if (prop != null)
            {
                var fmt = prop.GetCustomAttribute<JsonSchemaFormatAttribute>()?.Format;
                if (fmt != null) result["format"] = fmt;
                var enumAttr = prop.GetCustomAttribute<JsonSchemaEnumAttribute>();
                if (enumAttr != null)
                    result["enum"] = enumAttr.Values.Select(v => (object?)v).ToList();
            }
            return result;
        }

        if (type == typeof(long))
            return new Dictionary<string, object?> { ["type"] = "integer" };
        if (type == typeof(ulong))
            return new Dictionary<string, object?> { ["type"] = "integer" };
        if (type == typeof(double))
            return new Dictionary<string, object?> { ["type"] = "number" };
        if (type == typeof(bool))
            return new Dictionary<string, object?> { ["type"] = "boolean" };
        if (type == typeof(DateTimeOffset))
            return new Dictionary<string, object?> { ["type"] = "string", ["format"] = "date-time" };
        if (type == typeof(JsonElement))
            return new Dictionary<string, object?>();

        // List<T>
        if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(List<>))
        {
            var itemType = type.GetGenericArguments()[0];
            var result = new Dictionary<string, object?> { ["type"] = "array" };
            var itemSchema = TryConvertAsRef(itemType) ?? ConvertType(itemType);

            if (IsNullableType(itemType))
            {
                var innerItemType = Nullable.GetUnderlyingType(itemType)!;
                var nonNullSchema = TryConvertAsRef(innerItemType) ?? ConvertType(innerItemType);
                itemSchema = new Dictionary<string, object?>
                {
                    ["anyOf"] = new List<object?> { nonNullSchema, new Dictionary<string, object?> { ["type"] = "null" } }
                };
            }

            result["items"] = itemSchema;
            return result;
        }

        // Dictionary<string, T>
        if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Dictionary<,>))
        {
            var valType = type.GetGenericArguments()[1];
            var result = new Dictionary<string, object?> { ["type"] = "object" };
            if (valType != typeof(JsonElement))
                result["additionalProperties"] = TryConvertAsRef(valType) ?? ConvertType(valType);
            return result;
        }

        if (TypeToTitle.TryGetValue(type, out var title))
            return new Dictionary<string, object?> { ["$ref"] = title };

        return new Dictionary<string, object?>();
    }

    // -----------------------------------------------------------------------
    // Wrapper detection and conversion
    // -----------------------------------------------------------------------

    private bool IsArrayWrapper(Type type, PropertyInfo[] props)
    {
        return props.Length == 1 && props[0].Name == "Items" &&
               props[0].PropertyType.IsGenericType &&
               props[0].PropertyType.GetGenericTypeDefinition() == typeof(List<>);
    }

    private void ConvertArrayWrapper(Type type, PropertyInfo[] props, Dictionary<string, object?> result)
    {
        var itemsProp = props[0];
        var listType = itemsProp.PropertyType;
        var itemType = listType.GetGenericArguments()[0];
        result["type"] = "array";

        var itemsNullable = itemsProp.GetCustomAttribute<JsonSchemaItemsNullableAttribute>() != null;
        var itemsRange = itemsProp.GetCustomAttribute<JsonSchemaItemsRangeAttribute>();

        if (itemsNullable)
        {
            var actualItemType = itemType.IsGenericType && itemType.GetGenericTypeDefinition() == typeof(Nullable<>)
                ? Nullable.GetUnderlyingType(itemType)!
                : itemType;
            var nonNullSchema = TryConvertAsRef(actualItemType) ?? ConvertType(actualItemType);

            if (itemsRange != null)
            {
                if (itemsRange.Minimum != null)
                    nonNullSchema["minimum"] = ParseJsonNumber(itemsRange.Minimum);
                if (itemsRange.Maximum != null)
                    nonNullSchema["maximum"] = ParseJsonNumber(itemsRange.Maximum);
            }

            result["items"] = new Dictionary<string, object?>
            {
                ["anyOf"] = new List<object?> { nonNullSchema, new Dictionary<string, object?> { ["type"] = "null" } }
            };
        }
        else
        {
            var itemSchema = TryConvertAsRef(itemType) ?? ConvertType(itemType);

            if (IsNullableType(itemType))
            {
                var innerType = Nullable.GetUnderlyingType(itemType)!;
                var nonNullSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType);
                itemSchema = new Dictionary<string, object?>
                {
                    ["anyOf"] = new List<object?> { nonNullSchema, new Dictionary<string, object?> { ["type"] = "null" } }
                };
            }

            if (itemsRange != null)
            {
                if (itemsRange.Minimum != null)
                    ((Dictionary<string, object?>)itemSchema)["minimum"] = ParseJsonNumber(itemsRange.Minimum);
                if (itemsRange.Maximum != null)
                    ((Dictionary<string, object?>)itemSchema)["maximum"] = ParseJsonNumber(itemsRange.Maximum);
            }

            result["items"] = itemSchema;
        }

        var range = itemsProp.GetCustomAttribute<JsonSchemaRangeAttribute>();
        if (range != null)
        {
            if (range.Minimum != null) result["minItems"] = ParseJsonNumber(range.Minimum);
            if (range.Maximum != null) result["maxItems"] = ParseJsonNumber(range.Maximum);
        }
    }

    private bool IsValueWrapper(Type type, PropertyInfo[] props)
    {
        if (props.Length != 1 || props[0].Name != "Value") return false;
        if (props[0].GetCustomAttribute<JsonPropertyNameAttribute>() != null) return false;
        if (props[0].GetCustomAttribute<JsonSchemaNullableAttribute>() != null) return false;
        return true;
    }

    private void ConvertValueWrapper(Type type, PropertyInfo[] props, Dictionary<string, object?> result)
    {
        var prop = props[0];
        var propType = prop.PropertyType;
        var typeSchema = TryConvertAsRef(propType) ?? ConvertType(propType, prop);
        result.MergeFrom(typeSchema);
        AddConstraints(result, prop);
    }

    private void ConvertNullableWrapper(Type type, PropertyInfo prop, Dictionary<string, object?> result)
    {
        var propType = prop.PropertyType;
        var innerType = Nullable.GetUnderlyingType(propType) ?? propType;
        var nonNullSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType, prop);
        AddConstraints(nonNullSchema, prop);
        result["anyOf"] = new List<object?>
        {
            nonNullSchema,
            new Dictionary<string, object?> { ["type"] = "null" }
        };
    }

    // -----------------------------------------------------------------------
    // Constraints
    // -----------------------------------------------------------------------

    private void AddConstraints(Dictionary<string, object?> schema, PropertyInfo prop)
    {
        var range = prop.GetCustomAttribute<JsonSchemaRangeAttribute>();
        if (range != null)
        {
            if (range.Minimum != null) schema["minimum"] = ParseJsonNumber(range.Minimum);
            if (range.Maximum != null) schema["maximum"] = ParseJsonNumber(range.Maximum);
        }

        var regex = prop.GetCustomAttribute<RegularExpressionAttribute>();
        if (regex != null) schema["pattern"] = regex.Pattern;

        var fmt = prop.GetCustomAttribute<JsonSchemaFormatAttribute>();
        if (fmt != null) schema["format"] = fmt.Format;

        var enumAttr = prop.GetCustomAttribute<JsonSchemaEnumAttribute>();
        if (enumAttr != null)
            schema["enum"] = enumAttr.Values.Select(v => (object?)v).ToList();

        var apSchemaAttr = prop.GetCustomAttribute<JsonSchemaAdditionalPropertiesSchemaAttribute>();
        if (apSchemaAttr != null)
            AddAdditionalPropertiesSchema(schema, apSchemaAttr);
    }

    private void AddArrayItemConstraints(Dictionary<string, object?> schema, PropertyInfo prop)
    {
        // Handle dictionary value constraints
        if (schema.ContainsKey("type") && schema["type"] as string == "object"
            && schema.ContainsKey("additionalProperties")
            && schema["additionalProperties"] is Dictionary<string, object?> apDict)
        {
            var dictRange = prop.GetCustomAttribute<JsonSchemaItemsRangeAttribute>();
            if (dictRange != null)
            {
                if (dictRange.Minimum != null) apDict["minimum"] = ParseJsonNumber(dictRange.Minimum);
                if (dictRange.Maximum != null) apDict["maximum"] = ParseJsonNumber(dictRange.Maximum);
            }
            return;
        }

        if (!schema.ContainsKey("type") || schema["type"] as string != "array") return;

        var itemsNullable = prop.GetCustomAttribute<JsonSchemaItemsNullableAttribute>() != null;
        var itemsRange = prop.GetCustomAttribute<JsonSchemaItemsRangeAttribute>();

        if (itemsNullable && schema.TryGetValue("items", out var currentItems) && currentItems is Dictionary<string, object?> currentItemsDict)
        {
            var nonNullSchema = new Dictionary<string, object?>(currentItemsDict);
            if (itemsRange != null)
            {
                if (itemsRange.Minimum != null) nonNullSchema["minimum"] = ParseJsonNumber(itemsRange.Minimum);
                if (itemsRange.Maximum != null) nonNullSchema["maximum"] = ParseJsonNumber(itemsRange.Maximum);
            }
            schema["items"] = new Dictionary<string, object?>
            {
                ["anyOf"] = new List<object?> { nonNullSchema, new Dictionary<string, object?> { ["type"] = "null" } }
            };
        }
        else if (itemsRange != null && schema.TryGetValue("items", out var items) && items is Dictionary<string, object?> itemsDict)
        {
            var target = itemsDict;
            while (target.TryGetValue("type", out var iType) && iType as string == "array"
                && target.TryGetValue("items", out var innerItems) && innerItems is Dictionary<string, object?> innerDict)
                target = innerDict;

            if (itemsRange.Minimum != null) target["minimum"] = ParseJsonNumber(itemsRange.Minimum);
            if (itemsRange.Maximum != null) target["maximum"] = ParseJsonNumber(itemsRange.Maximum);
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    private static void AddAdditionalPropertiesSchema(Dictionary<string, object?> result, JsonSchemaAdditionalPropertiesSchemaAttribute attr)
    {
        if (attr.Schema == "true")
            result["additionalProperties"] = true;
        else if (attr.Schema.StartsWith("$ref:"))
            result["additionalProperties"] = new Dictionary<string, object?> { ["$ref"] = attr.Schema[5..] };
        else if (attr.Schema.StartsWith("{") || attr.Schema.StartsWith("["))
        {
            // Complex JSON schema (inline anyOf, etc.)
            var el = JsonDocument.Parse(attr.Schema).RootElement;
            result["additionalProperties"] = JsonElementToStructure(el);
        }
    }

    /// <summary>
    /// Convert a JsonElement to the Dictionary/List/string/number structure used by the harness.
    /// </summary>
    private static object? JsonElementToStructure(JsonElement el)
    {
        return el.ValueKind switch
        {
            JsonValueKind.Object => el.EnumerateObject().ToDictionary(p => p.Name, p => JsonElementToStructure(p.Value)),
            JsonValueKind.Array => el.EnumerateArray().Select(JsonElementToStructure).ToList<object?>(),
            JsonValueKind.String => el.GetString(),
            JsonValueKind.Number => ParseJsonNumberFromElement(el),
            JsonValueKind.True => true,
            JsonValueKind.False => false,
            JsonValueKind.Null => null,
            _ => null,
        };
    }

    private static object ParseJsonNumberFromElement(JsonElement el)
    {
        var raw = el.GetRawText();
        if (raw.Contains('.') || raw.Contains('e') || raw.Contains('E'))
            return el.GetDouble();
        if (el.TryGetInt64(out var l)) return l;
        if (el.TryGetUInt64(out var ul)) return (long)ul;
        return el.GetDouble();
    }

    private static bool IsDictionaryType(Type type)
    {
        return type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Dictionary<,>);
    }

    private static bool IsNullableType(Type type)
    {
        return type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Nullable<>);
    }

    private static object ParseJsonNumber(string s)
    {
        if (s.Contains('.') || s.Contains('e') || s.Contains('E'))
            return double.Parse(s);
        if (long.TryParse(s, out var l))
            return l;
        if (ulong.TryParse(s, out var ul))
            return ul;
        return double.Parse(s);
    }

    private static object? ParseJsonDefault(string json)
    {
        var el = JsonDocument.Parse(json).RootElement;
        return el.ValueKind switch
        {
            JsonValueKind.String => el.GetString(),
            JsonValueKind.Number => ParseJsonNumber(el.GetRawText()),
            JsonValueKind.True => true,
            JsonValueKind.False => false,
            JsonValueKind.Null => null,
            _ => json,
        };
    }
}

internal static class DictExtensions
{
    internal static void MergeFrom(this Dictionary<string, object?> target, Dictionary<string, object?> source)
    {
        foreach (var kv in source)
            target[kv.Key] = kv.Value;
    }
}
