namespace ObjectiveAI.Agent.Completions.Message;

public partial class DeveloperMessage
{
    public void Push(DeveloperMessage other)
    {
        Content.Push(other.Content);
    }
}
