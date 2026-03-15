namespace ObjectiveAI.Attributes;

/// <summary>
/// Marks a type as having additionalProperties set explicitly in JSON Schema.
/// </summary>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Struct, Inherited = false)]
public sealed class JsonSchemaAdditionalPropertiesAttribute(bool allowed) : Attribute
{
    public bool Allowed { get; } = allowed;
}
