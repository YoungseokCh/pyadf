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

def parse_adf_str(json: str) -> ParsedAdf: ...
def parse_adf_dict(adf_dict: Any) -> ParsedAdf: ...
def render_markdown(
    parsed: ParsedAdf,
    config: MarkdownConfig | None = None,
) -> str: ...
def document_to_markdown(
    json: str,
    config: MarkdownConfig | None = None,
) -> str: ...
def convert_jsonl_batch(
    data: bytes,
    config: MarkdownConfig | None = None,
) -> list[tuple[str | None, str | None]]: ...
