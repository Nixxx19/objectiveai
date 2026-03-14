namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores the JSON Schema "type" value on a variant class when the variant
/// has both a $ref and a type (e.g., type: "object").
/// </summary>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class JsonSchemaVariantTypeAttribute(string type) : Attribute
{
    public string Type { get; } = type;
}
