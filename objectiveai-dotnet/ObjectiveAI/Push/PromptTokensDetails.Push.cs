using static ObjectiveAI.PushUtils;

namespace ObjectiveAI.Agent.Completions.Response;

public partial class PromptTokensDetails
{
    public void Push(PromptTokensDetails other)
    {
        AudioTokens = PushOptionUlong(AudioTokens, other.AudioTokens);
        CachedTokens = PushOptionUlong(CachedTokens, other.CachedTokens);
        CacheWriteTokens = PushOptionUlong(CacheWriteTokens, other.CacheWriteTokens);
        VideoTokens = PushOptionUlong(VideoTokens, other.VideoTokens);
    }
}
