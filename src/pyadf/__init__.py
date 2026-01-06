"""pyadf - A Python library for converting Atlassian Document Format (ADF) to Markdown."""

from ._logger import set_debug_mode
from .document import Document
from .exceptions import (
    InvalidADFError,
    InvalidFieldError,
    InvalidInputError,
    InvalidJSONError,
    MissingFieldError,
    NodeCreationError,
    PyADFError,
    UnsupportedNodeTypeError,
)
from .markdown import MarkdownConfig

__version__ = "0.1.0"
__all__ = [
    "Document",
    "MarkdownConfig",
    "set_debug_mode",
    "PyADFError",
    "InvalidADFError",
    "InvalidJSONError",
    "InvalidInputError",
    "MissingFieldError",
    "InvalidFieldError",
    "UnsupportedNodeTypeError",
    "NodeCreationError",
]
