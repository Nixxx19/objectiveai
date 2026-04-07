namespace ObjectiveAI.Vector.Completions.Response;

public partial class Vote
{
    // Vote is a leaf type with no mutable streaming fields.
    // New votes are appended to the parent's Votes list; individual
    // Vote objects are never merged.
}
