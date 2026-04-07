using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Functions.Inventions.Recursive.Response.Streaming;

public partial class FunctionInventionRecursiveChunk
{
    public void Push(FunctionInventionRecursiveChunk other)
    {
        PushByIndex(
            Inventions,
            other.Inventions,
            c => (long)c.Index,
            (a, b) => a.Push(b)
        );
        InventionsErrors = PushLazySetTrue(InventionsErrors, other.InventionsErrors);
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // id, created, object: immutable
    }
}
