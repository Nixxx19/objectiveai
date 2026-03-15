namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores a $ref target title on a variant class or property for roundtrip reconstruction.
/// </summary>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaRefAttribute(string refTitle) : Attribute
{
    public string RefTitle { get; } = refTitle;
}
