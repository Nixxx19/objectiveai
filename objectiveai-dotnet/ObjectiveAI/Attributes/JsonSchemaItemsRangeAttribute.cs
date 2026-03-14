namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores JSON Schema minimum/maximum constraints for array item types.
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaItemsRangeAttribute : Attribute
{
    public string? Minimum { get; set; }
    public string? Maximum { get; set; }
}
