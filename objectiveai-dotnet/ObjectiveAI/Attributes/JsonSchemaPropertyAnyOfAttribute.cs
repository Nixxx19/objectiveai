namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores the serialized anyOf JSON for a property whose type can't fully represent
/// the inline anyOf structure (e.g., non-nullable anyOf with multiple variants).
/// The roundtrip test uses this to reconstruct the anyOf instead of deriving from the C# type.
/// </summary>
[AttributeUsage(AttributeTargets.Property, Inherited = false)]
public sealed class JsonSchemaPropertyAnyOfAttribute(string json) : Attribute
{
    /// <summary>Serialized JSON of the anyOf array.</summary>
    public string Json { get; } = json;
}
