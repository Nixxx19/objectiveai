def __getattr__(name):
    from objectiveai._rebuild import ensure_rebuilt
    ensure_rebuilt()
    import importlib
    _generated = importlib.import_module(__name__ + '._generated')
    return getattr(_generated, name)


import objectiveai.agent.completions.response.streaming.agent_completion_chunk_methods  # noqa: F401, E402
import objectiveai.agent.completions.response.streaming.assistant_response_chunk_methods  # noqa: F401, E402
import objectiveai.agent.completions.response.streaming.message_chunk_methods  # noqa: F401, E402
from objectiveai.agent.completions.response.streaming.pyo3 import *  # noqa: F401, F403, E402
