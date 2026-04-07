using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Vector.Completions.Response.Streaming;

public partial class VectorCompletionChunk
{
    public void Push(VectorCompletionChunk other)
    {
        PushByIndex(
            Completions,
            other.Completions,
            c => (long)c.Index,
            (a, b) => a.Push(b)
        );
        Votes.AddRange(other.Votes);
        // scores: always replace
        Scores.Clear();
        Scores.AddRange(other.Scores);
        // weights: always replace
        Weights.Clear();
        Weights.AddRange(other.Weights);
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // id, created, swarm, object: immutable
    }
}
