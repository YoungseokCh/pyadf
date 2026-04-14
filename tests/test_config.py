"""Tests for MarkdownConfig options."""

import pytest

from pyadf import Document, MarkdownConfig


def _bullet_list_adf() -> dict:
    return {
        "type": "bulletList",
        "content": [
            {
                "type": "listItem",
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Item"}]}],
            }
        ],
    }


class TestBulletMarker:
    def test_default_is_dash(self):
        assert Document(_bullet_list_adf()).to_markdown() == "- Item"

    def test_asterisk(self):
        config = MarkdownConfig(bullet_marker="*")
        assert Document(_bullet_list_adf()).to_markdown(config) == "* Item"

    def test_dash(self):
        config = MarkdownConfig(bullet_marker="-")
        assert Document(_bullet_list_adf()).to_markdown(config) == "- Item"

    def test_invalid_raises(self):
        with pytest.raises(ValueError, match="Invalid bullet_marker"):
            MarkdownConfig(bullet_marker="x")


class TestShowLinks:
    def test_default_is_true(self):
        assert MarkdownConfig().show_links is True

    def test_can_disable_link_targets(self):
        assert MarkdownConfig(show_links=False).show_links is False
