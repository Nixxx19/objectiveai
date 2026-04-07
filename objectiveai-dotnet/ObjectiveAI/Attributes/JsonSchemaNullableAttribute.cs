namespace ObjectiveAI.Attributes;

/// <summary>
/// Marks a property as nullable in JSON Schema (anyOf with null variant).
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaNullableAttribute : Attribute
{
}
