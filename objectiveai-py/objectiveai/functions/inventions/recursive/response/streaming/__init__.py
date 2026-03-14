def __getattr__(name):
    from objectiveai._rebuild import ensure_rebuilt
    ensure_rebuilt()
    import importlib
    _generated = importlib.import_module(__name__ + '._generated')
    return getattr(_generated, name)


import objectiveai.functions.inventions.recursive.response.streaming.function_invention_chunk_methods  # noqa: F401, E402
import objectiveai.functions.inventions.recursive.response.streaming.function_invention_recursive_chunk_methods  # noqa: F401, E402
