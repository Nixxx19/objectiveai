using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Agent.Completions.Response.Streaming;

public partial class AssistantResponseChunk
{
    public void Push(AssistantResponseChunk other)
    {
        Reasoning = PushOptionString(Reasoning, other.Reasoning);

        if (ToolCalls != null && other.ToolCalls != null)
        {
            PushByIndex(
                ToolCalls,
                other.ToolCalls,
                t => (long)t.Index,
                (a, b) => a.Push(b)
            );
        }
        else if (other.ToolCalls != null)
        {
            ToolCalls = new List<Message.AssistantToolCallDelta>(other.ToolCalls);
        }

        var content = Content;
        PushOption(ref content, other.Content, (a, b) => a.Push(b));
        Content = content;

        Refusal = PushOptionString(Refusal, other.Refusal);
        FinishReason ??= other.FinishReason;

        var logprobs = Logprobs;
        PushOption(ref logprobs, other.Logprobs, (a, b) => a.Push(b));
        Logprobs = logprobs;

        if (string.IsNullOrEmpty(UpstreamId) && !string.IsNullOrEmpty(other.UpstreamId))
            UpstreamId = other.UpstreamId;

        ServiceTier ??= other.ServiceTier;
        SystemFingerprint ??= other.SystemFingerprint;
        Provider ??= other.Provider;

        var usage = Usage;
        PushOption(ref usage, other.Usage, (a, b) => a.Push(b));
        Usage = usage;
        // role, index, created, agent, model: immutable
    }
}
