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

    @pytest.mark.xfail(strict=True, reason="strict reject policy not enforced yet")
    @pytest.mark.parametrize(
        "markdown",
        [
            "[x][ref]\n\n[ref]: http://example.com",
            "`code`",
            "~~x~~",
            "- [ ] task",
            '<div adf="extension">',
            '<div adf="extension" params=\'{"bad":}\'></div>',
        ],
    )
    def test_known_reject_gaps(self, markdown):
        with pytest.raises(MarkdownParseError):
            Document.from_markdown(markdown)
