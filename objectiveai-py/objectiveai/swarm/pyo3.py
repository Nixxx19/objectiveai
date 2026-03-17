"""PyO3 bindings for swarm operations."""

import objectiveai_pyo3


def pyo3_validate_swarm(swarm):
    """Validate an swarm configuration and compute its content-addressed ID."""
    return objectiveai_pyo3.validate_swarm(swarm)
