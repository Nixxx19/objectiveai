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
        var assembly = typeof(ObjectiveAI.Attributes.JsonSchemaTitleAttribute).Assembly;
        foreach (var type in assembly.GetExportedTypes())
        {
            var attr = type.GetCustomAttribute<JsonSchemaTitleAttribute>();
            if (attr != null)
            {
                map[attr.Title] = type;
            }
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
        else if (type.IsInterface)
        {
            ConvertTopLevelInterface(type, result);
        }
        else
        {
            ConvertTopLevelClass(type, result);
        }

        return result;
    }

    private void ConvertTopLevelEnum(Type type, Dictionary<string, object?> result)
    {
        // Check if enum members have descriptions → anyOf pattern
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

    private void ConvertTopLevelInterface(Type type, Dictionary<string, object?> result)
    {
        // Check for stored $ref targets (marker interface pattern)
        var refsAttr = type.GetCustomAttribute<JsonSchemaAnyOfRefsAttribute>();
        if (refsAttr != null && refsAttr.Refs.Length > 0)
        {
            var anyOf = new List<object?>();
            foreach (var refTitle in refsAttr.Refs)
            {
                anyOf.Add(new Dictionary<string, object?> { ["$ref"] = refTitle });
            }
            result["anyOf"] = anyOf;
            return;
        }

        // Find variant types (general anyOf interface pattern)
        var variants = FindVariantTypes(type);

        if (variants.Count > 0)
        {
            var anyOf = new List<object?>();
            foreach (var variant in variants)
            {
                anyOf.Add(ConvertVariantType(variant));
            }
            result["anyOf"] = anyOf;
        }
    }

    private void ConvertTopLevelClass(Type type, Dictionary<string, object?> result)
    {
        // Check for variant types (flattened model pattern)
        var variants = FindVariantTypes(type);

        if (type.GetProperties().Length == 0 && variants.Count == 0)
        {
            // Empty object type
            result["type"] = "object";
            return;
        }

        // Check if this is a wrapper type (single "Value" property + no [JsonPropertyName])
        var props = type.GetProperties(BindingFlags.Public | BindingFlags.Instance);
        if (IsArrayWrapper(type, props))
        {
            ConvertArrayWrapper(type, props, result);
            return;
        }
        if (IsValueWrapper(type, props))
        {
            ConvertValueWrapper(type, props, result);
            return;
        }

        // Regular object type
        result["type"] = "object";

        if (variants.Count > 0)
        {
            // Check if this is a flattened $ref model (single variant with $ref, no anyOf)
            if (variants.Count == 1)
            {
                var singleVariant = variants[0];
                var singleRefAttr = singleVariant.GetCustomAttribute<JsonSchemaRefAttribute>();
                var singleProps = singleVariant.GetProperties(BindingFlags.Public | BindingFlags.Instance);
                bool isSingleValueRef = singleRefAttr != null
                    && singleProps.Length == 1 && singleProps[0].Name == "Value"
                    && singleProps[0].GetCustomAttribute<JsonPropertyNameAttribute>() == null;
                if (isSingleValueRef)
                {
                    // Direct $ref on top level (not anyOf)
                    result["$ref"] = singleRefAttr!.RefTitle;
                }
                else
                {
                    var anyOf = new List<object?>();
                    anyOf.Add(ConvertVariantType(singleVariant));
                    result["anyOf"] = anyOf;
                }
            }
            else
            {
                var anyOf = new List<object?>();
                foreach (var variant in variants)
                {
                    anyOf.Add(ConvertVariantType(variant));
                }
                result["anyOf"] = anyOf;
            }
        }

        var properties = ConvertProperties(type);
        if (properties.Count > 0)
            result["properties"] = properties;

        var additionalProps = type.GetCustomAttribute<JsonSchemaAdditionalPropertiesAttribute>();
        if (additionalProps != null)
            result["additionalProperties"] = additionalProps.Allowed;
    }

    // -----------------------------------------------------------------------
    // Property conversion
    // -----------------------------------------------------------------------

    private Dictionary<string, object?> ConvertProperties(Type type)
    {
        var result = new Dictionary<string, object?>();
        foreach (var prop in type.GetProperties(BindingFlags.Public | BindingFlags.Instance))
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

        // Description
        var desc = prop.GetCustomAttribute<DescriptionAttribute>()?.Description;
        if (desc != null)
            result["description"] = desc;

        var propType = prop.PropertyType;
        var isNullable = prop.GetCustomAttribute<JsonSchemaNullableAttribute>() != null;
        var innerType = isNullable
            ? Nullable.GetUnderlyingType(propType) ?? propType
            : propType;

        if (!isNullable)
        {
            // Non-nullable property
            var typeSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType, prop);
            result.MergeFrom(typeSchema);
            AddConstraints(result, prop);
            AddArrayItemConstraints(result, prop);
        }
        else
        {
            // Nullable property → anyOf: [nonNullSchema, {type: "null"}]
            var nonNullSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType, prop);
            AddConstraints(nonNullSchema, prop);
            AddArrayItemConstraints(nonNullSchema, prop);

            result["anyOf"] = new List<object?>
            {
                nonNullSchema,
                new Dictionary<string, object?> { ["type"] = "null" }
            };
        }

        // Default value
        var defaultAttr = prop.GetCustomAttribute<JsonSchemaDefaultAttribute>();
        if (defaultAttr != null)
        {
            result["default"] = ParseJsonDefault(defaultAttr.JsonValue);
        }

        return result;
    }

    // -----------------------------------------------------------------------
    // Type conversion
    // -----------------------------------------------------------------------

    private Dictionary<string, object?>? TryConvertAsRef(Type type)
    {
        if (TypeToTitle.TryGetValue(type, out var title))
        {
            return new Dictionary<string, object?> { ["$ref"] = title };
        }
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
                if (fmt != null)
                    result["format"] = fmt;

                // Check for enum values
                var enumAttr = prop.GetCustomAttribute<JsonSchemaEnumAttribute>();
                if (enumAttr != null)
                {
                    result["enum"] = enumAttr.Values.Select(v => (object?)v).ToList();
                }
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
            return new Dictionary<string, object?>(); // bare schema

        // List<T>
        if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(List<>))
        {
            var itemType = type.GetGenericArguments()[0];
            var result = new Dictionary<string, object?> { ["type"] = "array" };

            var itemSchema = TryConvertAsRef(itemType) ?? ConvertType(itemType);

            // Check for Nullable<T> items (value type)
            if (IsNullableType(itemType))
            {
                var innerItemType = Nullable.GetUnderlyingType(itemType)!;
                var nonNullSchema = TryConvertAsRef(innerItemType) ?? ConvertType(innerItemType);
                itemSchema = new Dictionary<string, object?>
                {
                    ["anyOf"] = new List<object?>
                    {
                        nonNullSchema,
                        new Dictionary<string, object?> { ["type"] = "null" }
                    }
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
            {
                result["additionalProperties"] = TryConvertAsRef(valType) ?? ConvertType(valType);
            }
            return result;
        }

        // Known type
        if (TypeToTitle.TryGetValue(type, out var title))
        {
            return new Dictionary<string, object?> { ["$ref"] = title };
        }

        return new Dictionary<string, object?>();
    }

    // -----------------------------------------------------------------------
    // Variant types
    // -----------------------------------------------------------------------

    private List<Type> FindVariantTypes(Type type)
    {
        var variants = new List<Type>();
        var assembly = type.Assembly;
        var baseName = type.Name;

        // For interfaces, strip the 'I' prefix
        if (type.IsInterface && baseName.StartsWith("I"))
            baseName = baseName[1..];

        int i = 1;
        while (true)
        {
            var variantName = $"{baseName}Variant{i}";
            var variantType = assembly.GetTypes().FirstOrDefault(t => t.Name == variantName && t.Namespace == type.Namespace);
            if (variantType == null)
                break;
            variants.Add(variantType);
            i++;
        }
        return variants;
    }

    private Dictionary<string, object?> ConvertVariantType(Type variant)
    {
        var result = new Dictionary<string, object?>();

        // Description
        var desc = variant.GetCustomAttribute<DescriptionAttribute>()?.Description;
        if (desc != null)
            result["description"] = desc;

        // Check for $ref + type + properties pattern (e.g., Message variants)
        var refAttr = variant.GetCustomAttribute<JsonSchemaRefAttribute>();
        var variantTypeAttr = variant.GetCustomAttribute<JsonSchemaVariantTypeAttribute>();

        var props = variant.GetProperties(BindingFlags.Public | BindingFlags.Instance);

        // Check if this is a value wrapper (single "Value" property with no [JsonPropertyName])
        if (props.Length == 1 && props[0].Name == "Value" && props[0].GetCustomAttribute<JsonPropertyNameAttribute>() == null)
        {
            var valProp = props[0];
            var isValNullable = valProp.GetCustomAttribute<JsonSchemaNullableAttribute>() != null;

            if (isValNullable)
            {
                // Nullable value variant → anyOf: [{nonNullSchema}, {type: "null"}]
                var valType = valProp.PropertyType;
                var innerType = Nullable.GetUnderlyingType(valType) ?? valType;
                var nonNullSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType, valProp);
                AddConstraints(nonNullSchema, valProp);
                result["anyOf"] = new List<object?>
                {
                    nonNullSchema,
                    new Dictionary<string, object?> { ["type"] = "null" }
                };
                return result;
            }

            if (refAttr != null)
            {
                // $ref variant (with possible description)
                result["$ref"] = refAttr.RefTitle;
            }
            else
            {
                var valType = valProp.PropertyType;
                var typeSchema = TryConvertAsRef(valType) ?? ConvertType(valType, valProp);
                result.MergeFrom(typeSchema);
            }

            // Add constraints from the Value property
            AddConstraints(result, valProp);
            AddArrayItemConstraints(result, valProp);

            return result;
        }

        // Variant with properties
        if (variantTypeAttr != null)
        {
            // Has explicit type (e.g., "object")
            result["type"] = variantTypeAttr.Type;
        }
        else if (refAttr == null)
        {
            // No $ref, regular object variant
            result["type"] = "object";
        }

        if (refAttr != null)
        {
            result["$ref"] = refAttr.RefTitle;
        }

        // Check for additionalProperties: false
        var additionalProps = variant.GetCustomAttribute<JsonSchemaAdditionalPropertiesAttribute>();
        if (additionalProps != null)
            result["additionalProperties"] = additionalProps.Allowed;

        // Check for additionalProperties schema (true or $ref)
        var apSchema = variant.GetCustomAttribute<JsonSchemaAdditionalPropertiesSchemaAttribute>();
        if (apSchema != null)
        {
            if (apSchema.Schema == "true")
            {
                result["additionalProperties"] = true;
            }
            else if (apSchema.Schema.StartsWith("$ref:"))
            {
                var apRefTitle = apSchema.Schema[5..];
                result["additionalProperties"] = new Dictionary<string, object?> { ["$ref"] = apRefTitle };
            }
        }

        var properties = ConvertProperties(variant);
        if (properties.Count > 0)
            result["properties"] = properties;

        return result;
    }

    // -----------------------------------------------------------------------
    // Helpers
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
                ["anyOf"] = new List<object?>
                {
                    nonNullSchema,
                    new Dictionary<string, object?> { ["type"] = "null" }
                }
            };
        }
        else
        {
            var itemSchema = TryConvertAsRef(itemType) ?? ConvertType(itemType);

            // Check for Nullable<T> items (value type)
            if (IsNullableType(itemType))
            {
                var innerType = Nullable.GetUnderlyingType(itemType)!;
                var nonNullSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType);
                itemSchema = new Dictionary<string, object?>
                {
                    ["anyOf"] = new List<object?>
                    {
                        nonNullSchema,
                        new Dictionary<string, object?> { ["type"] = "null" }
                    }
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

        // minItems/maxItems from range attribute on the Items property
        var range = itemsProp.GetCustomAttribute<JsonSchemaRangeAttribute>();
        if (range != null)
        {
            if (range.Minimum != null)
                result["minItems"] = ParseJsonNumber(range.Minimum);
            if (range.Maximum != null)
                result["maxItems"] = ParseJsonNumber(range.Maximum);
        }
    }

    private bool IsValueWrapper(Type type, PropertyInfo[] props)
    {
        if (props.Length != 1 || props[0].Name != "Value") return false;
        return props[0].GetCustomAttribute<JsonPropertyNameAttribute>() == null;
    }

    private void ConvertValueWrapper(Type type, PropertyInfo[] props, Dictionary<string, object?> result)
    {
        var prop = props[0];
        var propType = prop.PropertyType;
        var isNullable = IsNullableType(propType);

        if (isNullable)
        {
            var innerType = propType.IsGenericType && propType.GetGenericTypeDefinition() == typeof(Nullable<>)
                ? Nullable.GetUnderlyingType(propType)!
                : propType;

            var nonNullSchema = TryConvertAsRef(innerType) ?? ConvertType(innerType, prop);
            AddConstraints(nonNullSchema, prop);

            result["anyOf"] = new List<object?>
            {
                nonNullSchema,
                new Dictionary<string, object?> { ["type"] = "null" }
            };
        }
        else
        {
            var typeSchema = TryConvertAsRef(propType) ?? ConvertType(propType, prop);
            result.MergeFrom(typeSchema);
            AddConstraints(result, prop);
        }
    }

    private static bool IsNullableType(Type type)
    {
        if (type.IsGenericType && type.GetGenericTypeDefinition() == typeof(Nullable<>))
            return true;
        return false;
    }

    private void AddConstraints(Dictionary<string, object?> schema, PropertyInfo prop)
    {
        var range = prop.GetCustomAttribute<JsonSchemaRangeAttribute>();
        if (range != null)
        {
            if (range.Minimum != null)
                schema["minimum"] = ParseJsonNumber(range.Minimum);
            if (range.Maximum != null)
                schema["maximum"] = ParseJsonNumber(range.Maximum);
        }

        var regex = prop.GetCustomAttribute<RegularExpressionAttribute>();
        if (regex != null)
            schema["pattern"] = regex.Pattern;

        var fmt = prop.GetCustomAttribute<JsonSchemaFormatAttribute>();
        if (fmt != null)
            schema["format"] = fmt.Format;

        var enumAttr = prop.GetCustomAttribute<JsonSchemaEnumAttribute>();
        if (enumAttr != null)
            schema["enum"] = enumAttr.Values.Select(v => (object?)v).ToList();

        var apSchemaAttr = prop.GetCustomAttribute<JsonSchemaAdditionalPropertiesSchemaAttribute>();
        if (apSchemaAttr != null)
        {
            if (apSchemaAttr.Schema == "true")
                schema["additionalProperties"] = true;
            else if (apSchemaAttr.Schema.StartsWith("$ref:"))
                schema["additionalProperties"] = new Dictionary<string, object?> { ["$ref"] = apSchemaAttr.Schema[5..] };
        }
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
                if (dictRange.Minimum != null)
                    apDict["minimum"] = ParseJsonNumber(dictRange.Minimum);
                if (dictRange.Maximum != null)
                    apDict["maximum"] = ParseJsonNumber(dictRange.Maximum);
            }
            return;
        }

        // Only applies to array types
        if (!schema.ContainsKey("type") || schema["type"] as string != "array")
            return;

        var itemsNullable = prop.GetCustomAttribute<JsonSchemaItemsNullableAttribute>() != null;
        var itemsRange = prop.GetCustomAttribute<JsonSchemaItemsRangeAttribute>();

        if (itemsNullable && schema.TryGetValue("items", out var currentItems) && currentItems is Dictionary<string, object?> currentItemsDict)
        {
            // Wrap items in anyOf with null
            var nonNullSchema = new Dictionary<string, object?>(currentItemsDict);
            if (itemsRange != null)
            {
                if (itemsRange.Minimum != null)
                    nonNullSchema["minimum"] = ParseJsonNumber(itemsRange.Minimum);
                if (itemsRange.Maximum != null)
                    nonNullSchema["maximum"] = ParseJsonNumber(itemsRange.Maximum);
            }

            schema["items"] = new Dictionary<string, object?>
            {
                ["anyOf"] = new List<object?>
                {
                    nonNullSchema,
                    new Dictionary<string, object?> { ["type"] = "null" }
                }
            };
        }
        else if (itemsRange != null && schema.TryGetValue("items", out var items) && items is Dictionary<string, object?> itemsDict)
        {
            // Drill down through nested arrays to find the leaf items
            var target = itemsDict;
            while (target.TryGetValue("type", out var iType) && iType as string == "array"
                && target.TryGetValue("items", out var innerItems) && innerItems is Dictionary<string, object?> innerDict)
            {
                target = innerDict;
            }

            if (itemsRange.Minimum != null)
                target["minimum"] = ParseJsonNumber(itemsRange.Minimum);
            if (itemsRange.Maximum != null)
                target["maximum"] = ParseJsonNumber(itemsRange.Maximum);
        }
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
        {
            target[kv.Key] = kv.Value;
        }
    }
}
