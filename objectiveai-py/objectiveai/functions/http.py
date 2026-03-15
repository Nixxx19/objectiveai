"""HTTP functions for functions."""


async def list_functions(client):
    raise NotImplementedError


async def get_function(client, id):
    raise NotImplementedError


async def get_function_usage(client, id):
    raise NotImplementedError


async def list_function_profile_pairs(client):
    raise NotImplementedError


async def get_function_profile_pair_usage(client, id):
    raise NotImplementedError
