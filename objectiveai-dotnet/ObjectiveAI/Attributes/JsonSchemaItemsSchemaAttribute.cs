namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores the serialized JSON schema for array items when the items schema
/// contains inline anyOf that can't be represented by the C# generic type parameter.
/// </summary>
[AttributeUsage(AttributeTargets.Property, Inherited = false)]
public sealed class JsonSchemaItemsSchemaAttribute(string json) : Attribute
{
    /// <summary>Serialized JSON of the items schema object.</summary>
    public string Json { get; } = json;
}
