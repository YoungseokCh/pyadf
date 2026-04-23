"""Document class for ADF to Markdown conversion."""

import warnings
from typing import Literal

from . import _core
from .exceptions import InvalidInputError
from .markdown import MarkdownConfig

KnownUnsupportedMode = Literal["error", "skip", "warn", "html"]


class Document:
    """
    Document class for handling Atlassian Document Format (ADF).

    This class provides a clean interface for converting ADF to Markdown.
    ADF input is parsed and validated eagerly at construction time (input
    errors surface here). Handling of known unsupported nodes is selected
    later in to_markdown().

    Example:
        >>> doc = Document('{"type": "doc", "content": [...]}')
        >>> markdown_text = doc.to_markdown()

        >>> doc = Document({"type": "doc", "content": [...]})
        >>> markdown_text = doc.to_markdown()

        >>> doc = Document()  # Empty document
        >>> markdown_text = doc.to_markdown()  # Returns ""
    """

    def __init__(
        self,
        adf: str | dict | None = None,
    ) -> None:
        """
        Initialize a Document from ADF data.

        Parses and validates the ADF structure eagerly. All input-related
        errors (bad JSON, missing fields, unknown unsupported node types) are
        raised here so that to_markdown() applies only rendering-time policy
        for known unsupported nodes.

        Args:
            adf: ADF data as a JSON string, dict, or None for empty document.
                 Can be any ADF node type including "doc".
        Raises:
            InvalidJSONError: If adf is a string but not valid JSON
            InvalidInputError: If adf has invalid type
            UnsupportedNodeTypeError: If ADF contains unknown unsupported node types
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

    @classmethod
    def from_markdown(cls, markdown: str) -> "Document":
        if not isinstance(markdown, str):
            raise InvalidInputError(
                expected_type="str",
                actual_type=type(markdown).__name__,
            )

        doc = cls()
        doc._parsed = _core.parse_markdown_str(markdown)
        return doc

    def to_markdown(
        self,
        config: MarkdownConfig | None = None,
        *,
        on_known_unsupported: KnownUnsupportedMode = "warn",
    ) -> str:
        """
        Convert the ADF document to Markdown.

        Renders from the pre-parsed tree cached at construction time.
        Known unsupported nodes are handled according to
        ``on_known_unsupported`` at render time.

        Args:
            config: Optional markdown configuration options
            on_known_unsupported: How to handle known unsupported node types
                such as "extension": "error", "skip", "warn", or "html".

        Returns:
            Markdown representation of the ADF content. Returns empty string
            if the document is empty or if the root node is None.
        """
        if self._parsed is None:
            return ""

        if on_known_unsupported not in ("error", "skip", "warn", "html"):
            raise ValueError(
                "on_known_unsupported must be 'error', 'skip', 'warn', or 'html', "
                f"got {on_known_unsupported!r}"
            )

        rust_config = None
        if config is not None:
            rust_config = _core.MarkdownConfig(config.bullet_marker, config.show_links)

        markdown, warnings_info = _core.render_markdown(
            self._parsed, rust_config, on_known_unsupported
        )
        if on_known_unsupported == "warn":
            for node_type, node_path in warnings_info:
                warnings.warn(
                    f'Known unsupported node type "{node_type}" skipped at: {node_path}',
                    UserWarning,
                    stacklevel=2,
                )
        return markdown

    def to_adf(self) -> dict:
        if self._parsed is None:
            return {"type": "doc", "content": []}

        adf = _core.parsed_adf_to_dict(self._parsed)
        if not isinstance(adf, dict):  # pragma: no cover
            raise TypeError("parsed_adf_to_dict() must return dict")
        return adf
