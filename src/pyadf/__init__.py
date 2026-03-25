"""pyadf - A Python library for converting Atlassian Document Format (ADF) to Markdown."""

from __future__ import annotations

import io
from collections.abc import Iterator
from dataclasses import dataclass
from typing import TYPE_CHECKING, Literal

from . import _core
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

if TYPE_CHECKING:
    from typing import BinaryIO

__version__ = "0.4.1"
__all__ = [
    "Document",
    "MarkdownConfig",
    "ConversionError",
    "convert_jsonl",
    "PyADFError",
    "InvalidADFError",
    "InvalidJSONError",
    "InvalidInputError",
    "MissingFieldError",
    "InvalidFieldError",
    "UnsupportedNodeTypeError",
    "NodeCreationError",
]


@dataclass
class ConversionError:
    """Represents a failed document conversion in a JSONL batch."""

    line_number: int
    error: str
    raw_line: str


def convert_jsonl(
    source: str | bytes | BinaryIO,
    *,
    config: MarkdownConfig | None = None,
    on_error: Literal["raise", "skip", "include"] = "include",
    batch_size: int = 10_000,
) -> Iterator[str | ConversionError]:
    """Convert a JSONL source (one ADF document per line) to markdown strings.

    Args:
        source: File path, bytes, or binary file-like object containing JSONL data.
        config: Optional markdown rendering configuration.
        on_error: Error handling strategy:
            "raise"   - raise PyADFError on first error
            "skip"    - silently skip failed documents
            "include" - yield ConversionError objects for failed documents
        batch_size: Number of lines to process per Rust batch call.

    Yields:
        Markdown strings for successful conversions,
        or ConversionError objects when on_error="include".
    """
    if batch_size < 1:
        raise ValueError(f"batch_size must be >= 1, got {batch_size}")
    if on_error not in ("raise", "skip", "include"):
        raise ValueError(f"on_error must be 'raise', 'skip', or 'include', got {on_error!r}")

    rust_config = None
    if config is not None:
        rust_config = _core.MarkdownConfig(config.bullet_marker, config.show_links)

    if isinstance(source, str):
        stream: BinaryIO = open(source, "rb")  # noqa: SIM115
        should_close = True
    elif isinstance(source, bytes):
        stream = io.BytesIO(source)
        should_close = True
    else:
        stream = source
        should_close = False

    try:
        global_line_num = 0
        eof = False
        while not eof:
            non_blank_lines: list[bytes] = []
            line_numbers: list[int] = []

            for _ in range(batch_size):
                raw = stream.readline()
                if not raw:
                    eof = True
                    break
                global_line_num += 1
                stripped = raw.rstrip(b"\r\n")
                if stripped:
                    non_blank_lines.append(stripped)
                    line_numbers.append(global_line_num)

            if not non_blank_lines:
                continue

            batch_data = b"\n".join(non_blank_lines)
            results = _core.convert_jsonl_batch(batch_data, rust_config)

            for (markdown_text, error_msg), raw_bytes, line_num in zip(
                results, non_blank_lines, line_numbers
            ):
                if markdown_text is not None:
                    yield markdown_text
                elif error_msg is not None:
                    raw_line = raw_bytes.decode("utf-8", errors="replace")
                    if on_error == "raise":
                        # Re-parse through Document to get the typed exception
                        # (InvalidJSONError, UnsupportedNodeTypeError, etc.)
                        Document(raw_line)
                        # If Document didn't raise (shouldn't happen), fall back
                        raise PyADFError(error_msg)  # pragma: no cover
                    elif on_error == "skip":
                        continue
                    else:
                        yield ConversionError(
                            line_number=line_num,
                            error=error_msg,
                            raw_line=raw_line,
                        )
    finally:
        if should_close:
            stream.close()
