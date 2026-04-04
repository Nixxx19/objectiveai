using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Functions.Inventions.Recursive.Response.Streaming;

public partial class FunctionInventionChunk
{
    public void Push(FunctionInventionChunk other)
    {
        PushByIndex(
            Completions,
            other.Completions,
            c => (long)c.Index,
            (a, b) => a.Push(b)
        );
        State = PushReplace(State, other.State);
        Path = PushReplace(Path, other.Path);
        Function = PushReplace(Function, other.Function);
        Error = PushReplace(Error, other.Error);
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // id, created, object, index: immutable
    }
}
