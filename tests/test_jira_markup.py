"""Tests for Jira wiki markup <-> Markdown conversion."""

import pytest

from pyadf import jira_to_markdown, markdown_to_jira


class TestJiraToMarkdown:
    """Tests for jira_to_markdown conversion."""

    def test_empty_input(self):
        assert jira_to_markdown("") == ""

    def test_none_like_empty(self):
        assert jira_to_markdown("") == ""

    def test_plain_text_passthrough(self):
        assert jira_to_markdown("hello world") == "hello world"

    def test_bold(self):
        assert "**bold**" in jira_to_markdown("*bold*")

    def test_italic(self):
        assert "*italic*" in jira_to_markdown("_italic_")

    def test_header_h1(self):
        result = jira_to_markdown("h1. Title")
        assert result.startswith("#")
        assert "Title" in result

    def test_header_h3(self):
        result = jira_to_markdown("h3. Title")
        assert "###" in result

    def test_header_h6(self):
        result = jira_to_markdown("h6. Deep")
        assert "######" in result

    def test_inline_code(self):
        assert "`code`" in jira_to_markdown("{{code}}")

    def test_code_block_with_lang(self):
        result = jira_to_markdown("{code:python}print('hi'){code}")
        assert "```python" in result
        assert "print('hi')" in result

    def test_code_block_without_lang(self):
        result = jira_to_markdown("{code}some code{code}")
        assert "```" in result
        assert "some code" in result

    def test_noformat(self):
        result = jira_to_markdown("{noformat}raw text{noformat}")
        assert "```" in result
        assert "raw text" in result

    def test_blockquote(self):
        result = jira_to_markdown("bq. quoted text")
        assert "> quoted text" in result

    def test_quote_block(self):
        result = jira_to_markdown("{quote}line1\nline2{quote}")
        assert "> line1" in result
        assert "> line2" in result

    def test_citation(self):
        assert "<cite>source</cite>" in jira_to_markdown("??source??")

    def test_inserted_text(self):
        assert "<ins>added</ins>" in jira_to_markdown("+added+")

    def test_superscript(self):
        assert "<sup>up</sup>" in jira_to_markdown("^up^")

    def test_subscript(self):
        assert "<sub>down</sub>" in jira_to_markdown("~down~")

    def test_strikethrough_passthrough(self):
        result = jira_to_markdown("-struck-")
        assert "-struck-" in result

    def test_image_simple(self):
        result = jira_to_markdown("!image.png!")
        assert "![](image.png)" in result

    def test_image_with_alt(self):
        result = jira_to_markdown("!img.png|alt=My Image!")
        assert "![My Image](img.png)" in result

    def test_link(self):
        result = jira_to_markdown("[Click here|http://example.com]")
        assert "[Click here](http://example.com)" in result

    def test_colored_text(self):
        result = jira_to_markdown("{color:red}warning{color}")
        assert '<span style="color:red">warning</span>' in result

    def test_unordered_list_single(self):
        result = jira_to_markdown("* item")
        assert "- item" in result

    def test_unordered_list_nested(self):
        result = jira_to_markdown("** nested")
        assert "  - nested" in result

    def test_ordered_list(self):
        result = jira_to_markdown("# first")
        assert "1. first" in result

    def test_ordered_list_nested(self):
        result = jira_to_markdown("## second level")
        assert "  1. second level" in result

    def test_table_headers(self):
        result = jira_to_markdown("||Name||Age||\n|Alice|30|")
        assert "|Name|Age|" in result
        assert "---|" in result


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
            "!image.png!",
        ],
    )
    def test_jira_roundtrip_preserves_content(self, jira):
        """Convert Jira->MD->Jira and verify key content is preserved."""
        md = jira_to_markdown(jira)
        back = markdown_to_jira(md)
        # Extract the core content word to verify it survived
        for word in ["bold", "italic", "Heading", "inline", "Link", "image"]:
            if word in jira:
                assert word in back, f"Lost '{word}' in roundtrip: {jira!r} -> {md!r} -> {back!r}"


class TestEdgeCases:
    """Edge cases and combined formatting."""

    def test_multiple_headers(self):
        result = jira_to_markdown("h1. First\nh2. Second")
        assert "# First" in result
        assert "## Second" in result

    def test_mixed_list_types(self):
        result = jira_to_markdown("* bullet\n# numbered")
        assert "- bullet" in result
        assert "1. numbered" in result

    def test_code_block_multiline(self):
        jira = "{code:java}\npublic class Foo {\n}\n{code}"
        result = jira_to_markdown(jira)
        assert "```java" in result
        assert "public class Foo" in result

    def test_multiline_quote(self):
        result = jira_to_markdown("{quote}first\nsecond\nthird{quote}")
        lines = [l for l in result.split("\n") if l.startswith(">")]
        assert len(lines) == 3
