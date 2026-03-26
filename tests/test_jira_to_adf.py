"""Tests for Jira markup -> ADF -> Markdown pipeline via Document(format='jira')."""

import pytest

from pyadf import Document, MarkdownConfig


class TestHeaders:
    def test_h1(self):
        assert Document("h1. Title", format="jira").to_markdown() == "# Title"

    def test_h2(self):
        assert Document("h2. Title", format="jira").to_markdown() == "## Title"

    def test_h3(self):
        assert Document("h3. Title", format="jira").to_markdown() == "### Title"

    def test_h6(self):
        assert Document("h6. Title", format="jira").to_markdown() == "###### Title"


class TestInlineFormatting:
    def test_bold(self):
        assert Document("*bold*", format="jira").to_markdown() == "**bold**"

    def test_italic(self):
        assert Document("_italic_", format="jira").to_markdown() == "*italic*"

    def test_inline_code(self):
        assert Document("use {{code}}", format="jira").to_markdown() == "use `code`"

    def test_strikethrough(self):
        assert Document("-struck-", format="jira").to_markdown() == "~~struck~~"

    def test_underline(self):
        assert Document("+inserted+", format="jira").to_markdown() == "<ins>inserted</ins>"

    def test_superscript(self):
        assert Document("^sup^", format="jira").to_markdown() == "<sup>sup</sup>"

    def test_subscript(self):
        assert Document("~sub~", format="jira").to_markdown() == "<sub>sub</sub>"

    def test_citation(self):
        # Citation (??) maps to em (italic) in ADF
        assert Document("??cited??", format="jira").to_markdown() == "*cited*"


class TestLinks:
    def test_link_with_show_links(self):
        config = MarkdownConfig(show_links=True)
        result = Document("[Click|http://example.com]", format="jira").to_markdown(config)
        assert result == "[Click](http://example.com)"

    def test_link_without_show_links(self):
        result = Document("[Click|http://example.com]", format="jira").to_markdown()
        assert result == "[Click]"


class TestCodeBlocks:
    def test_code_block_with_language(self):
        jira = "{code:python}\nprint('hi')\n{code}"
        assert Document(jira, format="jira").to_markdown() == "```python\nprint('hi')\n```"

    def test_code_block_no_language(self):
        jira = "{code}\nsome code\n{code}"
        assert Document(jira, format="jira").to_markdown() == "```\nsome code\n```"

    def test_noformat_block(self):
        jira = "{noformat}\nraw text\n{noformat}"
        assert Document(jira, format="jira").to_markdown() == "```\nraw text\n```"


class TestBlockquotes:
    def test_bq_line(self):
        assert Document("bq. quoted text", format="jira").to_markdown() == "> quoted text"

    def test_quote_block(self):
        jira = "{quote}\nhello\nworld\n{quote}"
        result = Document(jira, format="jira").to_markdown()
        assert "> hello" in result
        assert "> world" in result


class TestLists:
    def test_bullet_list(self):
        jira = "* item 1\n* item 2"
        assert Document(jira, format="jira").to_markdown() == "+ item 1\n+ item 2"

    def test_ordered_list(self):
        jira = "# first\n# second"
        assert Document(jira, format="jira").to_markdown() == "1. first\n2. second"

    def test_nested_bullet_list(self):
        jira = "* a\n** b\n* c"
        result = Document(jira, format="jira").to_markdown()
        assert "+ a" in result
        assert "  + b" in result
        assert "+ c" in result

    def test_nested_ordered_list(self):
        jira = "# a\n## b\n# c"
        result = Document(jira, format="jira").to_markdown()
        assert "1. a" in result
        assert "  1. b" in result
        assert "2. c" in result

    def test_deeply_nested_list(self):
        jira = "* a\n** b\n*** c"
        result = Document(jira, format="jira").to_markdown()
        assert "+ a" in result
        assert "  + b" in result
        assert "    + c" in result


class TestTables:
    def test_header_and_data_rows(self):
        jira = "||Name||Age||\n|Alice|30|"
        result = Document(jira, format="jira").to_markdown()
        assert "| Name | Age |" in result
        assert "| --- | --- |" in result
        assert "| Alice | 30 |" in result


class TestMixedContent:
    def test_paragraph_header_list(self):
        jira = "Hello\n\nh2. Title\n\n* a\n* b"
        result = Document(jira, format="jira").to_markdown()
        assert "Hello" in result
        assert "## Title" in result
        assert "+ a" in result
        assert "+ b" in result


class TestEmptyAndEdgeCases:
    def test_empty_string(self):
        assert Document("", format="jira").to_markdown() == ""

    def test_none_input(self):
        assert Document(None, format="jira").to_markdown() == ""

    def test_format_jira_with_dict_raises(self):
        with pytest.raises(ValueError, match="format='jira' requires a string"):
            Document({}, format="jira")

    def test_invalid_format_raises(self):
        with pytest.raises(ValueError, match="Invalid format"):
            Document("text", format="xml")

    def test_default_format_is_adf(self):
        # dict input works with default format
        adf = {"type": "paragraph", "content": [{"type": "text", "text": "hi"}]}
        assert Document(adf).to_markdown() == "hi"
