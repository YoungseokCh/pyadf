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

        >>> doc = Document()  # Empty document
        >>> markdown_text = doc.to_markdown()  # Returns ""
    """

    def __init__(self, adf: str | dict | None = None) -> None:
        """
        Initialize a Document from ADF data.

        Parses and validates the ADF structure eagerly. All input-related
        errors (bad JSON, missing fields, unsupported node types) are raised
        here so that to_markdown() only performs rendering.

        Args:
            adf: ADF data as a JSON string, dict, or None for empty document.
                 Can be any ADF node type including "doc".

        Raises:
            InvalidJSONError: If adf is a string but not valid JSON
            InvalidInputError: If adf has invalid type
            UnsupportedNodeTypeError: If ADF contains unsupported node types
            MissingFieldError: If required fields are missing
            InvalidFieldError: If fields have invalid values
            NodeCreationError: If node creation fails
        """
        self._parsed: _core.ParsedAdf | None = None

        if adf is None:
            return

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
