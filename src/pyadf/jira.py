"""Jira wiki markup conversion utilities."""

from . import _core


def markdown_to_jira(text: str) -> str:
    """Convert Markdown to Jira wiki markup.

    Args:
        text: Text in Markdown format.

    Returns:
        Text converted to Jira wiki markup format.
    """
    if not text:
        return ""
    return _core.markdown_to_jira(text)
