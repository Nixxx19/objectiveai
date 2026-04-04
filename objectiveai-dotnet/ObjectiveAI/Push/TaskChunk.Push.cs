namespace ObjectiveAI.Functions.Executions.Response.Streaming;

public partial class TaskChunk
{
    public long? Index()
    {
        if (FunctionExecution != null)
            return (long)FunctionExecution.Index;
        if (VectorCompletion != null)
            return (long)VectorCompletion.Index;
        return null;
    }

    public void Push(TaskChunk other)
    {
        if (FunctionExecution != null && other.FunctionExecution != null)
            FunctionExecution.Push(other.FunctionExecution);
        else if (VectorCompletion != null && other.VectorCompletion != null)
            VectorCompletion.Push(other.VectorCompletion);
    }
}
