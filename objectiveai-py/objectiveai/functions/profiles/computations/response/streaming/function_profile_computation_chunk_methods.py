"""Methods for FunctionProfileComputationChunk."""
from __future__ import annotations

from objectiveai.push_utils import push_by_index, push_option, push_replace, push_lazy_set, push_lazy_set_true
from objectiveai.functions.profiles.computations.response.streaming.function_profile_computation_chunk import (
    FunctionProfileComputationChunk,
)


def _push(self, other: FunctionProfileComputationChunk) -> None:
    # executions: merge by index
    push_by_index(self.executions, other.executions)

    # executions_errors: lazy set true
    self.executions_errors = push_lazy_set_true(self.executions_errors, other.executions_errors)

    # profile: lazy set
    self.profile = push_lazy_set(self.profile, other.profile)

    # fitting_stats: lazy set
    self.fitting_stats = push_lazy_set(self.fitting_stats, other.fitting_stats)

    # retry_token: replace
    self.retry_token = push_replace(self.retry_token, other.retry_token)

    # usage: delegate
    self.usage = push_option(self.usage, other.usage)

    # id, created, object, function are immutable


FunctionProfileComputationChunk.push = _push
