namespace ObjectiveAI.Agent.Completions.Message;

public partial class SystemMessage
{
    public void Push(SystemMessage other)
    {
        Content.Push(other.Content);
    }
}
