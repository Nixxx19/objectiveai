namespace ObjectiveAI.Attributes;

/// <summary>
/// Maps a C# type back to its JSON Schema title for roundtrip reconstruction.
/// </summary>
[AttributeUsage(AttributeTargets.Class | AttributeTargets.Interface | AttributeTargets.Enum | AttributeTargets.Struct, Inherited = false)]
public sealed class JsonSchemaTitleAttribute(string title) : Attribute
{
    public string Title { get; } = title;
}
