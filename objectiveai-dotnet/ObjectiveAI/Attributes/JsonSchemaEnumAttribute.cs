namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores JSON Schema enum values on a property for roundtrip reconstruction.
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaEnumAttribute(params string[] values) : Attribute
{
    public string[] Values { get; } = values;
}
