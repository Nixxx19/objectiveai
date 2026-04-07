namespace ObjectiveAI.Agent.Completions.Response;

public partial class Logprobs
{
    public void Push(Logprobs other)
    {
        if (Content != null && other.Content != null)
            Content.AddRange(other.Content);
        else if (other.Content != null)
            Content = new List<Logprob>(other.Content);

        if (Refusal != null && other.Refusal != null)
            Refusal.AddRange(other.Refusal);
        else if (other.Refusal != null)
            Refusal = new List<Logprob>(other.Refusal);
    }
}
