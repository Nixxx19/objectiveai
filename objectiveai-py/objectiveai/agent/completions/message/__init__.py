def __getattr__(name):
    from objectiveai._rebuild import ensure_rebuilt
    ensure_rebuilt()
    import importlib
    _generated = importlib.import_module(__name__ + '._generated')
    return getattr(_generated, name)


import objectiveai.agent.completions.message.assistant_tool_call_delta_methods  # noqa: F401, E402
import objectiveai.agent.completions.message.assistant_tool_call_function_delta_methods  # noqa: F401, E402
import objectiveai.agent.completions.message.rich_content_methods  # noqa: F401, E402
