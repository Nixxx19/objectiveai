def __getattr__(name):
    import importlib
    _generated = importlib.import_module(__name__ + '._generated')
    return getattr(_generated, name)


import objectiveai.functions.executions.response.streaming.function_execution_chunk_methods  # noqa: F401, E402
import objectiveai.functions.executions.response.streaming.function_execution_task_chunk_methods  # noqa: F401, E402
import objectiveai.functions.executions.response.streaming.reasoning_summary_chunk_methods  # noqa: F401, E402
import objectiveai.functions.executions.response.streaming.task_chunk_methods  # noqa: F401, E402
import objectiveai.functions.executions.response.streaming.vector_completion_task_chunk_methods  # noqa: F401, E402
