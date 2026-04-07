using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Functions.Profiles.Computations.Response.Streaming;

public partial class FunctionProfileComputationChunk
{
    public void Push(FunctionProfileComputationChunk other)
    {
        PushByIndex(
            Executions,
            other.Executions,
            e => (long)e.Index,
            (a, b) => a.Push(b)
        );
        ExecutionsErrors = PushLazySetTrue(ExecutionsErrors, other.ExecutionsErrors);
        Profile = PushReplace(Profile, other.Profile);
        FittingStats = PushReplace(FittingStats, other.FittingStats);
        RetryToken = PushReplace(RetryToken, other.RetryToken);
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // id, created, object, function: immutable
    }
}
