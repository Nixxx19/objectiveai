using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Agent.Completions.Message;

public partial class AssistantToolCallFunctionDelta
{
    public void Push(AssistantToolCallFunctionDelta other)
    {
        Name ??= other.Name;
        Arguments = PushOptionString(Arguments, other.Arguments);
    }
}
