namespace ObjectiveAI.Agent.Completions.Response;

public partial class CostDetails
{
    public void Push(CostDetails other)
    {
        UpstreamInferenceCost += other.UpstreamInferenceCost;
        UpstreamUpstreamInferenceCost += other.UpstreamUpstreamInferenceCost;
    }
}
