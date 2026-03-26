"""Jira wiki markup <-> Markdown conversion."""

from . import _core


def jira_to_markdown(text: str) -> str:
    """Convert Jira wiki markup to Markdown.

    Args:
        text: Text in Jira wiki markup format.

    Returns:
        Text converted to Markdown format.
    """
    if not text:
        return ""
    return _core.jira_to_markdown(text)


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
