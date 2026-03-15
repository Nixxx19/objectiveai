namespace ObjectiveAI.Attributes;

/// <summary>
/// Marks array items as nullable in JSON Schema (items wrapped in anyOf with null variant).
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaItemsNullableAttribute : Attribute
{
}
