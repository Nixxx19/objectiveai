"""PyO3 bindings for ensemble operations."""

import objectiveai_pyo3


def pyo3_validate_ensemble(ensemble):
    """Validate an ensemble configuration and compute its content-addressed ID."""
    return objectiveai_pyo3.validate_ensemble(ensemble)
