"""pyadf - A Python library for converting Atlassian Document Format (ADF) to Markdown."""

from .adf2md import adf2md
from ._logger import set_debug_mode

__version__ = "0.1.0"
__all__ = ["adf2md", "set_debug_mode"]
