using ObjectiveAI.Agent.Completions.Response.Streaming;
using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Functions.Inventions.Response.Streaming;

public partial class AgentCompletionChunk
{
    public void Push(AgentCompletionChunk other)
    {
        PushByNullableIndex(
            Messages,
            other.Messages,
            m => m.Index(),
            (a, b) => a.Push(b)
        );
        Error = PushReplace(Error, other.Error);
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        Continuation = PushReplace(Continuation, other.Continuation);
        // id, created, object, upstream, index: immutable
    }
}
