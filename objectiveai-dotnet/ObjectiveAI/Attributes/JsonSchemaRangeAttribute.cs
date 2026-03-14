namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores JSON Schema minimum/maximum as exact string representations for lossless roundtrip.
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaRangeAttribute : Attribute
{
    public string? Minimum { get; set; }
    public string? Maximum { get; set; }
}
