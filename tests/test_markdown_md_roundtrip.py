"""Expanded Markdown -> ADF -> Markdown roundtrip tests."""

import pytest

from pyadf import Document


class TestMarkdownRoundtripExact:
    @pytest.mark.parametrize(
        "markdown",
        [
            "Hello",
            "# Title",
            "*x*",
            "**x**",
            "`code`",
            "~~x~~",
            "[`code`](http://e.com)",
            "[http://e.com](http://e.com)",
            "## **Title**",
            "Hello, **world!**",
            "A  \nB",
            "- **A**\n- B",
            "- [ ] task",
            "- [x] task",
            "1. A\n2. B\n3. C",
            "> Quote",
            "> A\n>\n> B",
            "> - A\n> - B",
            "> ```\n> x\n> ```",
            "```\na\n\nb\n```",
            "## **Title**",
        ],
    )
    def test_exact_roundtrip(self, markdown):
        assert Document.from_markdown(markdown).to_markdown(on_known_unsupported="html") == markdown

    def test_html_fallback_block_roundtrip(self):
        markdown = '<div adf="extension" params=\'{"extensionKey":"toc"}\'></div>'
        assert Document.from_markdown(markdown).to_markdown(on_known_unsupported="html") == markdown

    def test_html_fallback_inline_roundtrip(self):
        markdown = 'Before <span adf="extension" params=\'{"extensionKey":"toc"}\'></span>'
        assert Document.from_markdown(markdown).to_markdown(on_known_unsupported="html") == markdown


class TestMarkdownRoundtripCanonicalized:
    @pytest.mark.parametrize(
        ("markdown", "expected_markdown"),
        [
            ("_x_", "*x*"),
            ("__x__", "**x**"),
            ("***[x](http://e.com)***", "[***x***](http://e.com)"),
            ("<http://e.com>", "[http://e.com](http://e.com)"),
            ("```python linenos\nprint(1)\n```", "```python\nprint(1)\n```"),
        ],
    )
    def test_canonicalized_roundtrip(self, markdown, expected_markdown):
        assert (
            Document.from_markdown(markdown).to_markdown(on_known_unsupported="html")
            == expected_markdown
        )


class TestMarkdownRoundtripKnownGaps:
    def test_nested_list_roundtrip(self):
        markdown = "- A\n  - B"
        assert Document.from_markdown(markdown).to_markdown() == markdown

    def test_multi_paragraph_list_item_roundtrip(self):
        markdown = "- A\n\n  B"
        assert Document.from_markdown(markdown).to_markdown() == markdown

    def test_table_with_inline_marks_roundtrip(self):
        markdown = "| **A** | [B](http://e.com) |\n| --- | --- |\n| C | D |"
        assert Document.from_markdown(markdown).to_markdown() == markdown
