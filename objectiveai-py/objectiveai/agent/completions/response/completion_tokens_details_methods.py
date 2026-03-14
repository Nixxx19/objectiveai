"""Methods for CompletionTokensDetails."""
from __future__ import annotations

from objectiveai.push_utils import push_option_int
from objectiveai.agent.completions.response.completion_tokens_details import (
    AgentCompletionsResponseCompletionTokensDetails,
)


def _push(self, other: AgentCompletionsResponseCompletionTokensDetails) -> None:
    self.accepted_prediction_tokens = push_option_int(
        self.accepted_prediction_tokens, other.accepted_prediction_tokens,
    )
    self.audio_tokens = push_option_int(self.audio_tokens, other.audio_tokens)
    self.reasoning_tokens = push_option_int(self.reasoning_tokens, other.reasoning_tokens)
    self.rejected_prediction_tokens = push_option_int(
        self.rejected_prediction_tokens, other.rejected_prediction_tokens,
    )


AgentCompletionsResponseCompletionTokensDetails.push = _push
