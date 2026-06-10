"""Type stubs for the Rust native extension module."""

from typing import Any

class ParsedAdf:
    """Opaque handle to a parsed ADF tree."""

    ...

class MarkdownConfig:
    """Rust-side markdown configuration."""

    bullet_marker: str
    show_links: bool
    date_timezone: str
    date_format: str
    def __init__(
        self,
        bullet_marker: str = "-",
        show_links: bool = True,
        date_timezone: str = "UTC",
        date_format: str = "%Y-%m-%dT%H:%M:%S%:z",
    ) -> None: ...

def parse_adf_str(json: str) -> ParsedAdf: ...
def parse_adf_dict(adf_dict: Any) -> ParsedAdf: ...
def parse_markdown_str(markdown: str) -> ParsedAdf: ...
def parsed_adf_to_dict(parsed: ParsedAdf) -> dict[str, Any]: ...
def render_markdown(
    parsed: ParsedAdf,
    config: MarkdownConfig | None = None,
    on_known_unsupported: str = "warn",
) -> tuple[str, list[tuple[str, str]]]: ...
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
