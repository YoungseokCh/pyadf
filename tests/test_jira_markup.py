"""Tests for Jira wiki markup <-> Markdown conversion."""

import pytest

from pyadf import Document, MarkdownConfig, markdown_to_jira


def j2m(text: str, show_links: bool = False) -> str:
    """Convert Jira markup to Markdown via the Document(format='jira') path."""
    config = MarkdownConfig(show_links=show_links) if show_links else None
    return Document(text, format="jira").to_markdown(config)


class TestJiraToMarkdown:
    """Tests for Jira -> Markdown via Document(format='jira')."""

    def test_empty_input(self):
        assert Document("", format="jira").to_markdown() == ""

    def test_plain_text_passthrough(self):
        assert j2m("hello world") == "hello world"

    def test_bold(self):
        assert "**bold**" in j2m("*bold*")

    def test_italic(self):
        assert "*italic*" in j2m("_italic_")

    def test_header_h1(self):
        result = j2m("h1. Title")
        assert result.startswith("#")
        assert "Title" in result

    def test_header_h3(self):
        assert "###" in j2m("h3. Title")

    def test_header_h6(self):
        assert "######" in j2m("h6. Deep")

    def test_inline_code(self):
        assert "`code`" in j2m("{{code}}")

    def test_code_block_with_lang(self):
        result = j2m("{code:python}print('hi'){code}")
        assert "```python" in result
        assert "print('hi')" in result

    def test_code_block_without_lang(self):
        result = j2m("{code}some code{code}")
        assert "```" in result
        assert "some code" in result

    def test_noformat(self):
        result = j2m("{noformat}raw text{noformat}")
        assert "```" in result
        assert "raw text" in result

    def test_blockquote(self):
        result = j2m("bq. quoted text")
        assert "> quoted text" in result

    def test_quote_block(self):
        result = j2m("{quote}line1\nline2{quote}")
        assert "> line1" in result
        assert "> line2" in result

    def test_inserted_text(self):
        assert "<ins>added</ins>" in j2m("+added+")

    def test_superscript(self):
        assert "<sup>up</sup>" in j2m("^up^")

    def test_subscript(self):
        assert "<sub>down</sub>" in j2m("~down~")

    def test_strikethrough(self):
        assert "~~struck~~" in j2m("-struck-")

    def test_link(self):
        result = j2m("[Click here|http://example.com]", show_links=True)
        assert "[Click here](http://example.com)" in result

    def test_colored_text(self):
        result = j2m("{color:red}warning{color}")
        assert '<span style="color:red">warning</span>' in result

    def test_unordered_list_single(self):
        assert "item" in j2m("* item")

    def test_unordered_list_nested(self):
        result = j2m("** nested")
        assert "nested" in result

    def test_ordered_list(self):
        result = j2m("# first")
        assert "first" in result

    def test_table_headers(self):
        result = j2m("||Name||Age||\n|Alice|30|")
        assert "Name" in result
        assert "Age" in result
        assert "Alice" in result


class TestMarkdownToJira:
    """Tests for markdown_to_jira conversion."""

    def test_empty_input(self):
        assert markdown_to_jira("") == ""

    def test_plain_text_passthrough(self):
        assert markdown_to_jira("hello world") == "hello world"

    def test_bold(self):
        assert "*bold*" in markdown_to_jira("**bold**")

    def test_italic(self):
        assert "_italic_" in markdown_to_jira("*italic*")

    def test_atx_header_h1(self):
        result = markdown_to_jira("# Title")
        assert "h1." in result
        assert "Title" in result

    def test_atx_header_h3(self):
        result = markdown_to_jira("### Title")
        assert "h3." in result

    def test_setext_header_h1(self):
        result = markdown_to_jira("Title\n===")
        assert "h1." in result
        assert "Title" in result

    def test_setext_header_h2(self):
        result = markdown_to_jira("Title\n---")
        assert "h2." in result

    def test_inline_code(self):
        assert "{{code}}" in markdown_to_jira("`code`")

    def test_code_block_with_lang(self):
        result = markdown_to_jira("```python\nprint('hi')\n```")
        assert "{code:python}" in result
        assert "print('hi')" in result
        assert "{code}" in result

    def test_code_block_without_lang(self):
        result = markdown_to_jira("```\nsome code\n```")
        assert "{code}" in result
        assert "some code" in result

    def test_strikethrough(self):
        result = markdown_to_jira("~~struck~~")
        assert "-struck-" in result

    def test_cite_tag(self):
        result = markdown_to_jira("<cite>source</cite>")
        assert "??source??" in result

    def test_del_tag(self):
        result = markdown_to_jira("<del>removed</del>")
        assert "-removed-" in result

    def test_ins_tag(self):
        result = markdown_to_jira("<ins>added</ins>")
        assert "+added+" in result

    def test_sup_tag(self):
        result = markdown_to_jira("<sup>up</sup>")
        assert "^up^" in result

    def test_sub_tag(self):
        result = markdown_to_jira("<sub>down</sub>")
        assert "~down~" in result

    def test_image_no_alt(self):
        result = markdown_to_jira("![](image.png)")
        assert "!image.png!" in result

    def test_image_with_alt(self):
        result = markdown_to_jira("![My Image](img.png)")
        assert "!img.png|alt=My Image!" in result

    def test_link(self):
        result = markdown_to_jira("[Click](http://example.com)")
        assert "[Click|http://example.com]" in result

    def test_bare_link(self):
        result = markdown_to_jira("<http://example.com>")
        assert "[http://example.com]" in result

    def test_colored_text(self):
        result = markdown_to_jira('<span style="color:#ff0000">red</span>')
        assert "{color:#ff0000}red{color}" in result

    def test_bullet_list(self):
        result = markdown_to_jira("- item")
        assert "* item" in result

    def test_bullet_list_nested(self):
        result = markdown_to_jira("  - nested")
        assert "** nested" in result

    def test_numbered_list_nested(self):
        result = markdown_to_jira("    1. sub item")
        assert "## sub item" in result

    def test_table_header_conversion(self):
        result = markdown_to_jira("| Name | Age |\n|---|---|\n| Alice | 30 |")
        assert "||" in result
        assert "---|" not in result


class TestRoundTrip:
    """Test that conversions preserve semantics in round trips."""

    @pytest.mark.parametrize(
        "jira",
        [
            "*bold text*",
            "_italic text_",
            "h2. A Heading",
            "{{inline code}}",
            "[Link|http://example.com]",
        ],
    )
    def test_jira_roundtrip_preserves_content(self, jira):
        """Convert Jira->MD->Jira and verify key content is preserved."""
        md = j2m(jira, show_links=True)
        back = markdown_to_jira(md)
        for word in ["bold", "italic", "Heading", "inline", "Link"]:
            if word in jira:
                assert word in back, f"Lost '{word}' in roundtrip: {jira!r} -> {md!r} -> {back!r}"


class TestEdgeCases:
    """Edge cases and combined formatting."""

    def test_multiple_headers(self):
        result = j2m("h1. First\nh2. Second")
        assert "# First" in result
        assert "## Second" in result

    def test_code_block_multiline(self):
        jira = "{code:java}\npublic class Foo {\n}\n{code}"
        result = j2m(jira)
        assert "```java" in result
        assert "public class Foo" in result

    def test_multiline_quote(self):
        result = j2m("{quote}first\nsecond\nthird{quote}")
        lines = [line for line in result.split("\n") if line.startswith(">")]
        assert len(lines) >= 2
