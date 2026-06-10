"""Markdown configuration for ADF to Markdown conversion."""

from dataclasses import dataclass
from typing import Literal

BulletMarker = Literal["+", "-", "*"]


@dataclass
class MarkdownConfig:
    """Configuration options for markdown generation."""

    bullet_marker: BulletMarker = "-"
    show_links: bool = True
    date_timezone: str = "UTC"
    date_format: str = "%Y-%m-%dT%H:%M:%S%:z"

    def __post_init__(self) -> None:
        if self.bullet_marker not in ("+", "-", "*"):
            raise ValueError(f"Invalid bullet_marker: {self.bullet_marker!r}")
