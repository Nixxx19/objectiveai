namespace ObjectiveAI.Attributes;

/// <summary>
/// Marks a property as a variant in a union class.
/// Each variant property represents one entry in the schema's anyOf array.
/// </summary>
[AttributeUsage(AttributeTargets.Property, Inherited = false)]
public sealed class JsonSchemaVariantAttribute(string title) : Attribute
{
    /// <summary>The variant's title (from the "title" field in the anyOf entry).</summary>
    public string Title { get; } = title;

    /// <summary>If the variant references another schema: the $ref title.</summary>
    public string? Ref { get; set; }

    /// <summary>If the variant has a JSON type: "string", "object", "array", etc.</summary>
    public string? Type { get; set; }

    /// <summary>If the variant is a string enum: the allowed values.</summary>
    public string[]? Enum { get; set; }
}
