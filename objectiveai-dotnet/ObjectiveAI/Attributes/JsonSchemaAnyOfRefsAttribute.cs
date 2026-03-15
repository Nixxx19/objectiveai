namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores the $ref targets from an anyOf pattern on a marker interface.
/// </summary>
[AttributeUsage(AttributeTargets.Interface | AttributeTargets.Class, Inherited = false)]
public sealed class JsonSchemaAnyOfRefsAttribute(params string[] refs) : Attribute
{
    public string[] Refs { get; } = refs;
}
