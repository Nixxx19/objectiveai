namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores the JSON Schema "default" value as a JSON string for roundtrip reconstruction.
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaDefaultAttribute(string jsonValue) : Attribute
{
    /// <summary>The default value serialized as a JSON string (e.g. "false", "\"instruction\"").</summary>
    public string JsonValue { get; } = jsonValue;
}
