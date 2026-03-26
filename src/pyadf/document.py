"""Document class for Atlassian document format conversion."""

from . import _core
from .exceptions import InvalidInputError
from .markdown import MarkdownConfig

class Document:
    """Universal document class for Atlassian content formats.

    Accepts multiple input formats (ADF, Jira markup, Markdown, HTML) and
    normalizes to an ADF tree internally. All input is parsed eagerly at
    construction time; rendering via to_markdown() cannot fail due to bad input.

    Example:
        >>> doc = Document('{"type": "doc", "content": [...]}')
        >>> doc = Document({"type": "doc", "content": [...]})
        >>> doc = Document("h1. Hello", format="jira")
        >>> doc = Document("# Hello", format="markdown")
        >>> doc = Document("<h1>Hello</h1>", format="html")
        >>> doc.to_markdown()
    """

    _VALID_FORMATS = ("adf", "jira", "markdown", "html")

    def __init__(self, adf: str | dict | None = None, *, format: str = "adf") -> None:
        """Initialize a Document from content in any supported format.

        Args:
            adf: Input data as a string, dict (ADF only), or None.
            format: Input format -- "adf" (default), "jira", "markdown", or "html".

        Raises:
            ValueError: If format is invalid or dict used with non-ADF format.
            InvalidJSONError: If ADF string is not valid JSON.
            InvalidInputError: If input type is unsupported.
        """
        if format not in self._VALID_FORMATS:
            raise ValueError(
                f"Invalid format: {format!r}. Must be one of {self._VALID_FORMATS}"
            )

        self._parsed: _core.ParsedAdf | None = None

        if adf is None:
            return

        if format != "adf":
            if isinstance(adf, dict):
                raise ValueError(f"format={format!r} requires a string, not dict")
            if not isinstance(adf, str):
                raise InvalidInputError(
                    expected_type="str or None",
                    actual_type=type(adf).__name__,
                )
            if format == "jira":
                self._parsed = _core.parse_jira_str(adf)
            elif format == "markdown":
                self._parsed = _core.parse_markdown_str(adf)
            elif format == "html":
                self._parsed = _core.parse_html_str(adf)
            return

        # format == "adf"
        if isinstance(adf, str):
            self._parsed = _core.parse_adf_str(adf)
        elif isinstance(adf, dict):
            self._parsed = _core.parse_adf_dict(adf)
        else:
            raise InvalidInputError(
                expected_type="str, dict, or None",
                actual_type=type(adf).__name__,
            )

    def to_markdown(self, config: MarkdownConfig | None = None) -> str:
        """Convert the document to Markdown.

        Args:
            config: Optional markdown configuration options.

        Returns:
            Markdown string. Empty string if the document is empty.
        """
        if self._parsed is None:
            return ""

        rust_config = None
        if config is not None:
            rust_config = _core.MarkdownConfig(config.bullet_marker, config.show_links)

        return _core.render_markdown(self._parsed, rust_config)

    def to_adf(self) -> dict | None:
        """Serialize the document to an ADF JSON dict.

        Returns:
            ADF dict, or None if the document is empty.
        """
        if self._parsed is None:
            return None
        return _core.render_adf_json(self._parsed)

    def to_html(self) -> str:
        """Render the document to HTML.

        Returns:
            HTML string. Empty string if the document is empty.
        """
        if self._parsed is None:
            return ""
        return _core.render_html(self._parsed)

    def to_jira(self) -> str:
        """Render the document to Jira wiki markup.

        Returns:
            Jira markup string. Empty string if the document is empty.
        """
        if self._parsed is None:
            return ""
        return _core.render_jira(self._parsed)
