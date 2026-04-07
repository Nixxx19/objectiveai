namespace ObjectiveAI.Agent.Completions.Response.Streaming;

public partial class MessageChunk
{
    public long? Index()
    {
        if (Assistant != null)
            return (long)Assistant.Index;
        if (Tool != null)
            return (long)Tool.Index;
        return null;
    }

    public void Push(MessageChunk other)
    {
        if (Assistant != null && other.Assistant != null)
            Assistant.Push(other.Assistant);
    }
}
