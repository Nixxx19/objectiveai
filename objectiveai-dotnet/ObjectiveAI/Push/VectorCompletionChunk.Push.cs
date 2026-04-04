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
        if (other.Scores.Count > 0)
        {
            Scores.Clear();
            Scores.AddRange(other.Scores);
        }
        if (other.Weights.Count > 0)
        {
            Weights.Clear();
            Weights.AddRange(other.Weights);
        }
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // id, created, swarm, object: immutable
    }
}
