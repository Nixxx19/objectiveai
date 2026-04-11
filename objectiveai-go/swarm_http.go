package objectiveai

import "context"

func SwarmListSwarms(ctx context.Context, c *Client, params SwarmListSwarmsRequest) (*SwarmListSwarmResponse, error) {
	return GetUnary[SwarmListSwarmResponse](ctx, c, "swarms/list", params)
}

func SwarmGetSwarm(ctx context.Context, c *Client, params RemotePathCommitOptional) (*SwarmGetSwarmResponse, error) {
	return GetUnary[SwarmGetSwarmResponse](ctx, c, "swarms", params)
}

func SwarmGetSwarmUsage(ctx context.Context, c *Client, params RemotePathCommitOptional) (*SwarmUsageSwarmResponse, error) {
	return GetUnary[SwarmUsageSwarmResponse](ctx, c, "swarms/usage", params)
}
