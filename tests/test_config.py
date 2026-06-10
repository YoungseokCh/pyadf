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


class TestDateConfig:
    def _date_doc(self) -> dict:
        # 1582152559000 ms == 2020-02-19T22:49:19Z
        return {"type": "date", "attrs": {"timestamp": "1582152559000"}}

    def test_defaults(self):
        config = MarkdownConfig()
        assert config.date_timezone == "UTC"
        assert config.date_format == "%Y-%m-%dT%H:%M:%S%:z"

    def test_invalid_timezone_raises(self):
        config = MarkdownConfig(date_timezone="Mars/Olympus")
        with pytest.raises(ValueError, match="Invalid date_timezone"):
            Document(self._date_doc()).to_markdown(config)

    def test_invalid_date_format_raises(self):
        config = MarkdownConfig(date_format="%Q")
        with pytest.raises(ValueError, match="Invalid date_format"):
            Document(self._date_doc()).to_markdown(config)
