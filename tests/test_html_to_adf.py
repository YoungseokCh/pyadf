"""Tests for HTML/XHTML -> ADF -> Markdown pipeline via Document(format='html')."""

import pytest

from pyadf import Document, MarkdownConfig


class TestSimpleElements:
    def test_simple_paragraph(self):
        assert Document("<p>Hello</p>", format="html").to_markdown() == "Hello"

    def test_bold(self):
        html = "<p><strong>bold</strong></p>"
        assert Document(html, format="html").to_markdown() == "**bold**"

    def test_bold_b_tag(self):
        html = "<p><b>bold</b></p>"
        assert Document(html, format="html").to_markdown() == "**bold**"

    def test_italic(self):
        html = "<p><em>italic</em></p>"
        assert Document(html, format="html").to_markdown() == "*italic*"

    def test_italic_i_tag(self):
        html = "<p><i>italic</i></p>"
        assert Document(html, format="html").to_markdown() == "*italic*"

    def test_bold_and_italic(self):
        html = "<p><strong>bold</strong> and <em>italic</em></p>"
        assert Document(html, format="html").to_markdown() == "**bold** and *italic*"


class TestHeaders:
    def test_h1(self):
        assert Document("<h1>Title</h1>", format="html").to_markdown() == "# Title"

    def test_h2(self):
        assert Document("<h2>Title</h2>", format="html").to_markdown() == "## Title"

    def test_h3(self):
        assert Document("<h3>Title</h3>", format="html").to_markdown() == "### Title"

    def test_h4(self):
        assert Document("<h4>Title</h4>", format="html").to_markdown() == "#### Title"

    def test_h5(self):
        assert Document("<h5>Title</h5>", format="html").to_markdown() == "##### Title"

    def test_h6(self):
        assert Document("<h6>Title</h6>", format="html").to_markdown() == "###### Title"


class TestLinks:
    def test_link_with_show_links(self):
        config = MarkdownConfig(show_links=True)
        html = '<a href="http://example.com">click</a>'
        result = Document(html, format="html").to_markdown(config)
        assert result == "[click](http://example.com)"

    def test_link_without_show_links(self):
        html = '<a href="http://example.com">click</a>'
        result = Document(html, format="html").to_markdown()
        assert result == "[click]"


class TestCodeBlocks:
    def test_code_block_with_language(self):
        html = '<pre><code class="language-python">print("hi")</code></pre>'
        result = Document(html, format="html").to_markdown()
        assert result == '```python\nprint("hi")\n```'

    def test_code_block_class_shorthand(self):
        html = '<pre><code class="python">print("hi")</code></pre>'
        result = Document(html, format="html").to_markdown()
        assert result == '```python\nprint("hi")\n```'

    def test_code_block_no_language(self):
        html = "<pre><code>some code</code></pre>"
        result = Document(html, format="html").to_markdown()
        assert result == "```\nsome code\n```"

    def test_pre_standalone(self):
        html = "<pre>raw text</pre>"
        result = Document(html, format="html").to_markdown()
        assert result == "```\nraw text\n```"

    def test_inline_code(self):
        # Inline code produces a "code" mark on the text node.
        # The rendered output depends on the markdown renderer's mark support.
        html = "<p>use <code>foo()</code></p>"
        result = Document(html, format="html").to_markdown()
        assert "foo()" in result


class TestBlockquotes:
    def test_blockquote(self):
        html = "<blockquote><p>quote</p></blockquote>"
        assert Document(html, format="html").to_markdown() == "> quote"


class TestLists:
    def test_unordered_list(self):
        html = "<ul><li>A</li><li>B</li></ul>"
        assert Document(html, format="html").to_markdown() == "+ A\n+ B"

    def test_ordered_list(self):
        html = "<ol><li>A</li><li>B</li></ol>"
        assert Document(html, format="html").to_markdown() == "1. A\n2. B"


class TestTables:
    def test_simple_table(self):
        html = "<table><tr><th>H</th></tr><tr><td>C</td></tr></table>"
        result = Document(html, format="html").to_markdown()
        assert "| H |" in result
        assert "| --- |" in result
        assert "| C |" in result

    def test_table_with_colspan(self):
        html = '<table><tr><td colspan="2">wide</td></tr></table>'
        result = Document(html, format="html").to_markdown()
        assert "wide" in result


class TestInlineFormatting:
    def test_nested_bold_italic(self):
        html = "<p><strong><em>bold italic</em></strong></p>"
        assert Document(html, format="html").to_markdown() == "***bold italic***"

    def test_strikethrough_del(self):
        # Strikethrough produces a "strike" mark; text content is preserved
        html = "<p><del>struck</del></p>"
        result = Document(html, format="html").to_markdown()
        assert "struck" in result

    def test_strikethrough_s(self):
        html = "<p><s>struck</s></p>"
        result = Document(html, format="html").to_markdown()
        assert "struck" in result

    def test_underline_u(self):
        html = "<p><u>underlined</u></p>"
        result = Document(html, format="html").to_markdown()
        assert "underlined" in result

    def test_underline_ins(self):
        html = "<p><ins>underlined</ins></p>"
        result = Document(html, format="html").to_markdown()
        assert "underlined" in result

    def test_superscript(self):
        html = "<p><sup>sup</sup></p>"
        result = Document(html, format="html").to_markdown()
        assert "sup" in result

    def test_subscript(self):
        html = "<p><sub>sub</sub></p>"
        result = Document(html, format="html").to_markdown()
        assert "sub" in result

    def test_line_break(self):
        html = "<p>line1<br>line2</p>"
        result = Document(html, format="html").to_markdown()
        assert "line1" in result
        assert "line2" in result


class TestColorSpan:
    def test_color_span(self):
        html = '<p><span style="color:red">colored</span></p>'
        result = Document(html, format="html").to_markdown()
        assert "colored" in result


class TestTransparentContainers:
    def test_div_transparent(self):
        html = "<div><p>text</p></div>"
        assert Document(html, format="html").to_markdown() == "text"

    def test_section_transparent(self):
        html = "<section><p>text</p></section>"
        assert Document(html, format="html").to_markdown() == "text"

    def test_article_transparent(self):
        html = "<article><p>text</p></article>"
        assert Document(html, format="html").to_markdown() == "text"

    def test_span_without_style(self):
        html = "<p><span>plain</span></p>"
        assert Document(html, format="html").to_markdown() == "plain"


class TestConfluenceTags:
    def test_ac_macro_skipped(self):
        html = '<p>before</p><ac:structured-macro ac:name="toc"></ac:structured-macro><p>after</p>'
        result = Document(html, format="html").to_markdown()
        assert "before" in result
        assert "after" in result

    def test_ri_tag_skipped(self):
        html = "<p>text</p><ri:user ri:account-id='123'/>"
        result = Document(html, format="html").to_markdown()
        assert "text" in result


class TestEdgeCases:
    def test_empty_input(self):
        assert Document("", format="html").to_markdown() == ""

    def test_whitespace_only(self):
        assert Document("   ", format="html").to_markdown() == ""

    def test_none_input(self):
        assert Document(None, format="html").to_markdown() == ""

    def test_format_html_with_dict_raises(self):
        with pytest.raises(ValueError, match="format='html' requires a string"):
            Document({}, format="html")

    def test_invalid_format_raises(self):
        with pytest.raises(ValueError, match="Invalid format"):
            Document("text", format="xml")

    def test_multiple_paragraphs(self):
        html = "<p>First</p><p>Second</p>"
        result = Document(html, format="html").to_markdown()
        assert "First" in result
        assert "Second" in result
