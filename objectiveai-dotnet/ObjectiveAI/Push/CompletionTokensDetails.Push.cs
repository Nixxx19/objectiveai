using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Agent.Completions.Response;

public partial class CompletionTokensDetails
{
    public void Push(CompletionTokensDetails other)
    {
        AcceptedPredictionTokens = PushOptionUlong(AcceptedPredictionTokens, other.AcceptedPredictionTokens);
        AudioTokens = PushOptionUlong(AudioTokens, other.AudioTokens);
        ReasoningTokens = PushOptionUlong(ReasoningTokens, other.ReasoningTokens);
        RejectedPredictionTokens = PushOptionUlong(RejectedPredictionTokens, other.RejectedPredictionTokens);
    }
}
