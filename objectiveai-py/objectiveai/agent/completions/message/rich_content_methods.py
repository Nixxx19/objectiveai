"""Methods for RichContent."""
from __future__ import annotations

from objectiveai.agent.completions.message.rich_content import (
    AgentCompletionsMessageRichContent,
    AgentCompletionsMessageRichContentVariant1,
    AgentCompletionsMessageRichContentVariant2,
)
from objectiveai.agent.completions.message.rich_content_part import (
    AgentCompletionsMessageRichContentPart,
    AgentCompletionsMessageRichContentPartVariant1,
)


def _push(self, other: AgentCompletionsMessageRichContent) -> None:
    self_inner = self.root
    other_inner = other.root

    self_is_text = isinstance(self_inner, AgentCompletionsMessageRichContentVariant1)
    other_is_text = isinstance(other_inner, AgentCompletionsMessageRichContentVariant1)

    if self_is_text and other_is_text:
        # text + text → concatenate
        self_inner.root += other_inner.root
    elif self_is_text and not other_is_text:
        # text + parts → convert self to parts, extend
        text_part = AgentCompletionsMessageRichContentPartVariant1(
            text=self_inner.root, type="text",
        )
        parts = [AgentCompletionsMessageRichContentPart(root=text_part)]
        parts.extend(other_inner.root)
        self.root = AgentCompletionsMessageRichContentVariant2(root=parts)
    elif not self_is_text and other_is_text:
        # parts + text → append text as new part
        if other_inner.root:
            text_part = AgentCompletionsMessageRichContentPartVariant1(
                text=other_inner.root, type="text",
            )
            self_inner.root.append(AgentCompletionsMessageRichContentPart(root=text_part))
    else:
        # parts + parts → extend
        if other_inner.root:
            self_inner.root.extend(other_inner.root)


AgentCompletionsMessageRichContent.push = _push
