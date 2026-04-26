"""Tests for intentionally rejected Markdown inputs."""

import pytest

from pyadf import Document, MarkdownParseError


class TestMarkdownReject:
    @pytest.mark.parametrize(
        "markdown",
        [
            "<div>hello</div>",
            "---",
            "![alt](http://example.com/image.png)",
            '<p adf="extension" params=\'{"extensionKey":"toc"}\'></p>',
        ],
    )
    def test_rejects_unsupported_markdown(self, markdown):
        with pytest.raises(MarkdownParseError):
            Document.from_markdown(markdown)

    def test_rejects_reference_style_link(self):
        with pytest.raises(MarkdownParseError):
            Document.from_markdown("[x][ref]\n\n[ref]: http://example.com")

    @pytest.mark.parametrize(
        "markdown",
        [
            '<div adf="extension">',
            '<div adf="extension" params=\'{"bad":}\'></div>',
        ],
    )
    def test_rejects_malformed_html_fallback(self, markdown):
        with pytest.raises(MarkdownParseError):
            Document.from_markdown(markdown)
