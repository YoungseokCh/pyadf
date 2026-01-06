"""Internal logging utilities for pyadf."""

import logging
import sys

# Configure default logger
_logger: logging.Logger | None = None


def get_logger() -> logging.Logger:
    """Get or create the pyadf logger."""
    global _logger

    if _logger is None:
        _logger = logging.getLogger("pyadf")
        _logger.setLevel(logging.WARNING)

        # Add console handler if no handlers exist
        if not _logger.handlers:
            handler = logging.StreamHandler(sys.stderr)
            handler.setFormatter(
                logging.Formatter("%(levelname)s: %(message)s")
            )
            _logger.addHandler(handler)

    return _logger


def set_debug_mode(enabled: bool) -> None:
    """Enable or disable debug mode."""
    logger = get_logger()
    logger.setLevel(logging.DEBUG if enabled else logging.WARNING)
