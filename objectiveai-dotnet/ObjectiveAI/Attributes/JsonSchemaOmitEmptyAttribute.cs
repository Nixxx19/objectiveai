namespace ObjectiveAI.Attributes;

/// <summary>
/// Marks a property as having "omitempty": true in its JSON schema.
/// </summary>
[AttributeUsage(AttributeTargets.Property, Inherited = false)]
public sealed class JsonSchemaOmitEmptyAttribute : Attribute;
