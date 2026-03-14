"""Methods for RichContent."""
from __future__ import annotations

from objectiveai.agent.completions.message.rich_content import (
    RichContent,
    RichContentVariant1,
    RichContentVariant2,
)
from objectiveai.agent.completions.message.rich_content_part import (
    RichContentPart,
    RichContentPartVariant1,
)


def _push(self, other: RichContent) -> None:
    self_inner = self.root
    other_inner = other.root

    self_is_text = isinstance(self_inner, RichContentVariant1)
    other_is_text = isinstance(other_inner, RichContentVariant1)

    if self_is_text and other_is_text:
        # text + text → concatenate
        self_inner.root += other_inner.root
    elif self_is_text and not other_is_text:
        # text + parts → convert self to parts, extend
        text_part = RichContentPartVariant1(
            text=self_inner.root, type="text",
        )
        parts = [RichContentPart(root=text_part)]
        parts.extend(other_inner.root)
        self.root = RichContentVariant2(root=parts)
    elif not self_is_text and other_is_text:
        # parts + text → append text as new part
        if other_inner.root:
            text_part = RichContentPartVariant1(
                text=other_inner.root, type="text",
            )
            self_inner.root.append(RichContentPart(root=text_part))
    else:
        # parts + parts → extend
        if other_inner.root:
            self_inner.root.extend(other_inner.root)


RichContent.push = _push
