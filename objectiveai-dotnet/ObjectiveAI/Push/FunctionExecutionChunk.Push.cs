using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Functions.Executions.Response.Streaming;

public partial class FunctionExecutionChunk
{
    public void Push(FunctionExecutionChunk other)
    {
        PushByNullableIndex(
            Tasks,
            other.Tasks,
            t => t.Index(),
            (a, b) => a.Push(b)
        );
        TasksErrors = PushLazySetTrue(TasksErrors, other.TasksErrors);
        var reasoning = Reasoning;
        PushOption(ref reasoning, other.Reasoning, (a, b) => a.Push(b));
        Reasoning = reasoning;
        Output = PushReplace(Output, other.Output);
        RetryToken = PushReplace(RetryToken, other.RetryToken);
        Error = PushReplace(Error, other.Error);
        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // id, created, object, function, profile: immutable
    }
}
