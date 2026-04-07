using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Functions.Executions.Response.Streaming;

public partial class VectorCompletionTaskChunk
{
    public void Push(VectorCompletionTaskChunk other)
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
        Error = PushReplace(Error, other.Error);
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // id, created, object, swarm, index, task_index, task_path: immutable
    }
}
