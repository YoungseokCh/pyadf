"""Type stubs for the Rust native extension module."""

from typing import Any

class ParsedAdf:
    """Opaque handle to a parsed ADF tree."""

    ...

class MarkdownConfig:
    """Rust-side markdown configuration."""

    bullet_marker: str
    show_links: bool
    def __init__(self, bullet_marker: str = "+", show_links: bool = False) -> None: ...

def parse_adf_str(json: str, on_known_unsupported: str = "warn") -> ParsedAdf: ...
def parse_adf_dict(adf_dict: Any, on_known_unsupported: str = "warn") -> ParsedAdf: ...
def skipped_known_unsupported(parsed: ParsedAdf) -> list[tuple[str, str]]: ...
def render_markdown(
    parsed: ParsedAdf,
    config: MarkdownConfig | None = None,
) -> str: ...
def document_to_markdown(
    json: str,
    config: MarkdownConfig | None = None,
    on_known_unsupported: str = "warn",
) -> str: ...
def convert_jsonl_batch(
    data: bytes,
    config: MarkdownConfig | None = None,
    on_known_unsupported: str = "warn",
) -> list[tuple[str | None, str | None, list[tuple[str, str]]]]: ...
