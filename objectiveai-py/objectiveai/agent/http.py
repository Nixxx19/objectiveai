"""HTTP functions for agents."""


async def list_agents(client):
    raise NotImplementedError


async def get_agent(client, id):
    raise NotImplementedError


async def get_agent_usage(client, id):
    raise NotImplementedError
