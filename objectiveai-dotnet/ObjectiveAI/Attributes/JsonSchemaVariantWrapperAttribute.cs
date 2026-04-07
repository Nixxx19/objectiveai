namespace ObjectiveAI.Attributes;

/// <summary>
/// Marks a class as a discriminated variant wrapper that inherits from a $ref base type
/// and adds discriminator properties.
/// </summary>
[AttributeUsage(AttributeTargets.Class, Inherited = false)]
public sealed class JsonSchemaVariantWrapperAttribute(string refTitle) : Attribute
{
    /// <summary>The $ref title this wrapper extends.</summary>
    public string Ref { get; } = refTitle;

    /// <summary>The JSON type of this variant (e.g., "object").</summary>
    public string? Type { get; set; }
}
