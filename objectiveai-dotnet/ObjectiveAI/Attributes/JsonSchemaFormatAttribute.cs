namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores a JSON Schema "format" value (e.g. "uuid") for roundtrip reconstruction.
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaFormatAttribute(string format) : Attribute
{
    public string Format { get; } = format;
}
