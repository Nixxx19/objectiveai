"""HTTP functions for authentication."""


async def create_api_key(client, body):
    raise NotImplementedError


async def create_openrouter_byok_api_key(client, body):
    raise NotImplementedError


async def disable_api_key(client, body):
    raise NotImplementedError


async def delete_openrouter_byok_api_key(client):
    raise NotImplementedError


async def list_api_keys(client):
    raise NotImplementedError


async def get_openrouter_byok_api_key(client):
    raise NotImplementedError


async def get_credits(client):
    raise NotImplementedError
