namespace ObjectiveAI.Attributes;

/// <summary>
/// Stores additionalProperties schema as a JSON string on a variant class.
/// For additionalProperties: true, stores "true".
/// For additionalProperties: {$ref: "..."}, stores the $ref title.
/// </summary>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Property | AttributeTargets.Field, Inherited = false)]
public sealed class JsonSchemaAdditionalPropertiesSchemaAttribute(string schema) : Attribute
{
    /// <summary>
    /// "true" for additionalProperties: true,
    /// "$ref:title" for additionalProperties: {$ref: "title"},
    /// or other JSON for more complex schemas.
    /// </summary>
    public string Schema { get; } = schema;
}
