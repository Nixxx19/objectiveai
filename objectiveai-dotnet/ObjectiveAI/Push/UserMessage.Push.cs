namespace ObjectiveAI.Agent.Completions.Message;

public partial class UserMessage
{
    public void Push(UserMessage other)
    {
        Content.Push(other.Content);
    }
}
