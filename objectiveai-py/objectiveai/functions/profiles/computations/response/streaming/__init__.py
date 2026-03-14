def __getattr__(name):
    import importlib
    _generated = importlib.import_module(__name__ + '._generated')
    return getattr(_generated, name)


import objectiveai.functions.profiles.computations.response.streaming.function_execution_chunk_methods  # noqa: F401, E402
import objectiveai.functions.profiles.computations.response.streaming.function_profile_computation_chunk_methods  # noqa: F401, E402
