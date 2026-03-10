"""Document class for ADF to Markdown conversion."""

import json
from typing import cast

from . import markdown, nodes
from ._types import JSONObject
from .exceptions import InvalidInputError, InvalidJSONError
from .markdown import MarkdownConfig


class Document:
    """
    Document class for handling Atlassian Document Format (ADF).

    This class provides a clean interface for converting ADF to Markdown.

    Example:
        >>> doc = Document('{"type": "doc", "content": [...]}')
        >>> markdown_text = doc.to_markdown()

        >>> doc = Document({"type": "doc", "content": [...]})
        >>> markdown_text = doc.to_markdown()

        >>> doc = Document()  # Empty document
        >>> markdown_text = doc.to_markdown()  # Returns ""
    """

    def __init__(self, adf: str | JSONObject | None = None) -> None:
        """
        Initialize a Document from ADF data.

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
        self._root_node: nodes.Node | None = None

        if adf is None:
            # Empty document
            return

        raw_adf_input = cast(object, adf)

        # Handle string input (JSON)
        if isinstance(raw_adf_input, str):
            try:
                raw_adf: object = json.loads(raw_adf_input)
            except json.JSONDecodeError as e:
                # Extract position from error message if available
                position = None
                if hasattr(e, "pos"):
                    position = e.pos
                raise InvalidJSONError(json_error=str(e), position=position) from e
            adf_dict = self._validate_adf_object(raw_adf)
        elif isinstance(raw_adf_input, dict):
            adf_dict = self._validate_adf_object(cast(object, raw_adf_input))
        else:
            raise InvalidInputError(
                expected_type="str, dict, or None",
                actual_type=type(raw_adf_input).__name__,
            )

        # Create node from the dict
        self._root_node = nodes.create_node_from_dict(adf_dict)

    @staticmethod
    def _validate_adf_object(value: object) -> JSONObject:
        if not isinstance(value, dict):
            raise InvalidInputError(
                expected_type="JSON object, dict, or None",
                actual_type=type(value).__name__,
            )

        return cast(JSONObject, value)

    def to_markdown(self, config: MarkdownConfig | None = None) -> str:
        """
        Convert the ADF document to Markdown.

        Args:
            config: Optional markdown configuration options

        Returns:
            Markdown representation of the ADF content. Returns empty string
            if the document is empty or if the root node is None.
        """
        if self._root_node is None:
            return ""

        return markdown.gen_md_from_root_node(self._root_node, config)
