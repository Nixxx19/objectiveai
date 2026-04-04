using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Agent.Completions.Message;

public partial class AssistantToolCallDelta
{
    public void Push(AssistantToolCallDelta other)
    {
        Type ??= other.Type;
        Id ??= other.Id;
        var function = Function;
        PushOption(ref function, other.Function, (a, b) => a.Push(b));
        Function = function;
    }
}
