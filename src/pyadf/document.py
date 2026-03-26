"""Document class for ADF to Markdown conversion."""

from . import _core
from .exceptions import InvalidInputError
from .markdown import MarkdownConfig


class Document:
    """
    Document class for handling Atlassian Document Format (ADF).

    This class provides a clean interface for converting ADF to Markdown.
    ADF input is parsed and validated eagerly at construction time (input
    errors surface here). Rendering from the cached tree in to_markdown()
    cannot fail due to bad input.

    Example:
        >>> doc = Document('{"type": "doc", "content": [...]}')
        >>> markdown_text = doc.to_markdown()

        >>> doc = Document({"type": "doc", "content": [...]})
        >>> markdown_text = doc.to_markdown()

        >>> doc = Document("h1. Hello", format="jira")
        >>> markdown_text = doc.to_markdown()  # Returns "# Hello"

        >>> doc = Document()  # Empty document
        >>> markdown_text = doc.to_markdown()  # Returns ""
    """

    _VALID_FORMATS = ("adf", "jira")

    def __init__(self, adf: str | dict | None = None, *, format: str = "adf") -> None:
        """
        Initialize a Document from ADF or Jira markup data.

        Parses and validates the input eagerly. All input-related errors are
        raised here so that to_markdown() only performs rendering.

        Args:
            adf: Input data as a JSON string, dict, Jira markup string, or None.
            format: Input format -- "adf" (default) or "jira".

        Raises:
            ValueError: If format is invalid or dict is used with format="jira"
            InvalidJSONError: If adf is a string in ADF mode but not valid JSON
            InvalidInputError: If adf has an unsupported type
            UnsupportedNodeTypeError: If ADF contains unsupported node types
            MissingFieldError: If required fields are missing
            InvalidFieldError: If fields have invalid values
        """
        if format not in self._VALID_FORMATS:
            raise ValueError(
                f"Invalid format: {format!r}. Must be one of {self._VALID_FORMATS}"
            )

        self._parsed: _core.ParsedAdf | None = None

        if adf is None:
            return

        if format == "jira":
            if isinstance(adf, dict):
                raise ValueError("format='jira' requires a string, not dict")
            if not isinstance(adf, str):
                raise InvalidInputError(
                    expected_type="str or None",
                    actual_type=type(adf).__name__,
                )
            self._parsed = _core.parse_jira_str(adf)
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
        """
        Convert the ADF document to Markdown.

        Renders from the pre-parsed tree cached at construction time.

        Args:
            config: Optional markdown configuration options

        Returns:
            Markdown representation of the ADF content. Returns empty string
            if the document is empty or if the root node is None.
        """
        if self._parsed is None:
            return ""

        rust_config = None
        if config is not None:
            rust_config = _core.MarkdownConfig(config.bullet_marker, config.show_links)

        return _core.render_markdown(self._parsed, rust_config)
