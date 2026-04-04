using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Agent.Completions.Response;

public partial class Usage
{
    public void Push(Usage other)
    {
        CompletionTokens += other.CompletionTokens;
        PromptTokens += other.PromptTokens;
        TotalTokens += other.TotalTokens;
        Cost += other.Cost;
        TotalCost += other.TotalCost;

        var completionTokensDetails = CompletionTokensDetails;
        PushOption(ref completionTokensDetails, other.CompletionTokensDetails, (a, b) => a.Push(b));
        CompletionTokensDetails = completionTokensDetails;

        var promptTokensDetails = PromptTokensDetails;
        PushOption(ref promptTokensDetails, other.PromptTokensDetails, (a, b) => a.Push(b));
        PromptTokensDetails = promptTokensDetails;

        var costDetails = CostDetails;
        PushOption(ref costDetails, other.CostDetails, (a, b) => a.Push(b));
        CostDetails = costDetails;
    }
}
