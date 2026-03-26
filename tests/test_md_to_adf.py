"""Tests for Markdown -> ADF parsing via Document(md, format="markdown")."""

import pytest

from pyadf import Document, MarkdownConfig


def md_to_md(md: str, *, show_links: bool = True) -> str:
    """Round-trip: markdown -> ADF -> markdown."""
    config = MarkdownConfig(bullet_marker="+", show_links=show_links)
    return Document(md, format="markdown").to_markdown(config)


class TestPlainText:
    def test_single_paragraph(self):
        assert md_to_md("Hello world") == "Hello world"

    def test_multiple_paragraphs(self):
        assert md_to_md("A\n\nB") == "A\n\nB"

    def test_empty_input(self):
        assert md_to_md("") == ""


class TestHeadings:
    def test_h1(self):
        assert md_to_md("# Title") == "# Title"

    def test_h2(self):
        assert md_to_md("## Title") == "## Title"

    def test_h3(self):
        assert md_to_md("### Title") == "### Title"

    def test_h4(self):
        assert md_to_md("#### Title") == "#### Title"

    def test_h5(self):
        assert md_to_md("##### Title") == "##### Title"

    def test_h6(self):
        assert md_to_md("###### Title") == "###### Title"


class TestInlineFormatting:
    def test_bold(self):
        assert md_to_md("**bold**") == "**bold**"

    def test_italic(self):
        assert md_to_md("*italic*") == "*italic*"

    def test_bold_italic(self):
        assert md_to_md("***both***") == "***both***"

    def test_inline_code(self):
        assert md_to_md("`code`") == "`code`"

    def test_strikethrough(self):
        assert md_to_md("~~struck~~") == "~~struck~~"


class TestLinks:
    def test_link_with_show_links(self):
        assert md_to_md("[click](http://example.com)") == "[click](http://example.com)"

    def test_link_without_show_links(self):
        result = md_to_md("[click](http://example.com)", show_links=False)
        assert result == "[click]"


class TestCodeBlocks:
    def test_fenced_with_language(self):
        md = "```python\nprint('hi')\n```"
        assert md_to_md(md) == "```python\nprint('hi')\n```"

    def test_fenced_without_language(self):
        md = "```\nhello\n```"
        assert md_to_md(md) == "```\nhello\n```"


class TestBlockquotes:
    def test_single_blockquote(self):
        assert md_to_md("> Quote") == "> Quote"

    def test_nested_blockquote(self):
        result = md_to_md("> > Nested")
        assert "> " in result
        assert "Nested" in result


class TestLists:
    def test_bullet_list(self):
        md = "- A\n- B"
        assert md_to_md(md) == "+ A\n+ B"

    def test_ordered_list(self):
        md = "1. A\n2. B"
        assert md_to_md(md) == "1. A\n2. B"

    def test_task_list(self):
        md = "- [ ] task one\n- [ ] task two"
        result = md_to_md(md)
        assert "task one" in result
        assert "task two" in result
        assert "[ ]" in result


class TestTables:
    def test_simple_table(self):
        md = "| Name | Age |\n| --- | --- |\n| Alice | 30 |"
        result = md_to_md(md)
        assert "Name" in result
        assert "Age" in result
        assert "Alice" in result
        assert "30" in result
        assert "---" in result


class TestHardBreaks:
    def test_hard_break(self):
        md = "line1  \nline2"
        result = md_to_md(md)
        assert "line1" in result
        assert "line2" in result


class TestMixedContent:
    def test_heading_paragraph_list_code(self):
        md = "# Title\n\nSome text.\n\n- item1\n- item2\n\n```python\ncode()\n```"
        result = md_to_md(md)
        assert "# Title" in result
        assert "Some text." in result
        assert "item1" in result
        assert "item2" in result
        assert "```python" in result
        assert "code()" in result


class TestRoundTrip:
    def test_structure_preserved(self):
        md = "# Header\n\nParagraph with **bold** and *italic*.\n\n- list item"
        result = md_to_md(md)
        assert "# Header" in result
        assert "**bold**" in result
        assert "*italic*" in result
        assert "list item" in result


class TestDocumentFormatParam:
    def test_format_markdown(self):
        doc = Document("Hello", format="markdown")
        assert doc.to_markdown() == "Hello"

    def test_format_adf_default(self):
        doc = Document('{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hi"}]}]}')
        assert doc.to_markdown() == "Hi"

    def test_format_invalid(self):
        with pytest.raises(ValueError, match="Invalid format"):
            Document("test", format="xml")

    def test_format_markdown_rejects_dict(self):
        with pytest.raises(ValueError, match="format='markdown' requires a string"):
            Document({"type": "doc"}, format="markdown")

    def test_format_markdown_none(self):
        doc = Document(None, format="markdown")
        assert doc.to_markdown() == ""


class TestInlineHTML:
    def test_underline(self):
        result = md_to_md("<ins>underlined</ins>")
        assert "<ins>" in result
        assert "underlined" in result

    def test_superscript(self):
        result = md_to_md("<sup>super</sup>")
        assert "<sup>" in result
        assert "super" in result

    def test_subscript(self):
        result = md_to_md("<sub>sub</sub>")
        assert "<sub>" in result
        assert "sub" in result

    def test_text_color(self):
        result = md_to_md('<span style="color:#ff0000">red</span>')
        assert "red" in result
        assert "color" in result
