"""Tests for Markdown -> ADF direct parsing and MD roundtrip."""

import pytest

from pyadf import Document, MarkdownParseError, markdown_to_adf


class TestToAdf:
    def test_existing_document_can_serialize_to_adf(self):
        adf = {
            "version": 1,
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Hello"}]},
            ],
        }

        assert Document(adf).to_adf() == adf


class TestFromMarkdown:
    def test_simple_paragraph(self):
        assert Document.from_markdown("Hello").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Hello"}]},
            ],
        }

    def test_heading(self):
        assert Document.from_markdown("# Title").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 1},
                    "content": [{"type": "text", "text": "Title"}],
                }
            ],
        }

    def test_strong_and_emphasis(self):
        assert Document.from_markdown("***Hi***").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "Hi",
                            "marks": [{"type": "strong"}, {"type": "em"}],
                        }
                    ],
                }
            ],
        }

    def test_link(self):
        assert Document.from_markdown("[x](http://example.com)").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {
                            "type": "text",
                            "text": "x",
                            "marks": [{"type": "link", "attrs": {"href": "http://example.com"}}],
                        }
                    ],
                }
            ],
        }

    def test_bullet_list(self):
        assert Document.from_markdown("- A\n- B").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "bulletList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "A"}],
                                }
                            ],
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "B"}],
                                }
                            ],
                        },
                    ],
                }
            ],
        }

    def test_ordered_list(self):
        assert Document.from_markdown("1. A\n2. B").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "orderedList",
                    "content": [
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "A"}],
                                }
                            ],
                        },
                        {
                            "type": "listItem",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "B"}],
                                }
                            ],
                        },
                    ],
                }
            ],
        }

    def test_hard_break(self):
        assert Document.from_markdown("A  \nB").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "A"},
                        {"type": "hardBreak"},
                        {"type": "text", "text": "B"},
                    ],
                }
            ],
        }

    def test_blockquote_two_paragraphs(self):
        assert Document.from_markdown("> A\n>\n> B").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "blockquote",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [{"type": "text", "text": "A"}],
                        },
                        {
                            "type": "paragraph",
                            "content": [{"type": "text", "text": "B"}],
                        },
                    ],
                }
            ],
        }

    def test_table(self):
        assert Document.from_markdown("| A | B |\n| --- | --- |\n| C | D |").to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "table",
                    "content": [
                        {
                            "type": "tableRow",
                            "content": [
                                {
                                    "type": "tableHeader",
                                    "content": [{"type": "text", "text": "A"}],
                                },
                                {
                                    "type": "tableHeader",
                                    "content": [{"type": "text", "text": "B"}],
                                },
                            ],
                        },
                        {
                            "type": "tableRow",
                            "content": [
                                {
                                    "type": "tableCell",
                                    "content": [{"type": "text", "text": "C"}],
                                },
                                {
                                    "type": "tableCell",
                                    "content": [{"type": "text", "text": "D"}],
                                },
                            ],
                        },
                    ],
                }
            ],
        }

    def test_markdown_to_adf_helper(self):
        assert markdown_to_adf("Hello") == Document.from_markdown("Hello").to_adf()

    def test_rejects_html(self):
        with pytest.raises(MarkdownParseError):
            Document.from_markdown("<div>hello</div>")

    def test_parses_block_html_fallback_for_known_unsupported(self):
        markdown = (
            '<div adf="extension" '
            'params=\'{"extensionKey":"toc","extensionType":"com.atlassian.confluence.macro.core"}\'></div>'
        )

        assert Document.from_markdown(markdown).to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "extension",
                    "attrs": {
                        "extensionKey": "toc",
                        "extensionType": "com.atlassian.confluence.macro.core",
                    },
                }
            ],
        }

    def test_parses_inline_html_fallback_for_known_unsupported(self):
        markdown = (
            'Before <span adf="extension" '
            'params=\'{"extensionKey":"toc","extensionType":"com.atlassian.confluence.macro.core"}\'></span>'
        )

        assert Document.from_markdown(markdown).to_adf() == {
            "version": 1,
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [
                        {"type": "text", "text": "Before "},
                        {
                            "type": "extension",
                            "attrs": {
                                "extensionKey": "toc",
                                "extensionType": "com.atlassian.confluence.macro.core",
                            },
                        },
                    ],
                }
            ],
        }


class TestRoundtrip:
    @pytest.mark.parametrize(
        "markdown",
        [
            "Hello",
            "# Title",
            "Hello, **world!**",
            "[x](http://example.com)",
            "A  \nB",
            "- A\n- B",
            "1. A\n2. B",
            "> Quote",
            "> A\n>\n> B",
            "```python\nprint('hello')\n```",
        ],
    )
    def test_markdown_roundtrip_for_canonical_subset(self, markdown):
        assert Document.from_markdown(markdown).to_markdown() == markdown

    def test_html_fallback_roundtrip_with_html_mode(self):
        markdown = '<div adf="extension" params=\'{"extensionKey":"toc"}\'></div>'
        assert Document.from_markdown(markdown).to_markdown(on_known_unsupported="html") == markdown

    def test_table_roundtrip_for_canonical_subset(self):
        markdown = "| A | B |\n| --- | --- |\n| C | D |"
        assert Document.from_markdown(markdown).to_markdown() == markdown
