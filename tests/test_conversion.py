"""Tests for ADF-to-Markdown conversion across all node types."""

import pytest

from pyadf import Document, MarkdownConfig, UnsupportedNodeTypeError


class TestParagraph:
    def test_simple_paragraph(self):
        adf = {
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Hello, world!"}]},
            ],
        }
        assert Document(adf).to_markdown() == "Hello, world!"

    def test_full_document_two_paragraphs(self):
        adf = {
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "First paragraph"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "Second paragraph"}]},
            ],
        }
        result = Document(adf).to_markdown()
        assert "First paragraph" in result
        assert "Second paragraph" in result

    def test_empty_document(self):
        assert Document().to_markdown() == ""

    def test_none_document(self):
        assert Document(None).to_markdown() == ""


class TestTextFormatting:
    def test_bold(self):
        adf = {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello, "},
                {"type": "text", "text": "world!", "marks": [{"type": "strong"}]},
            ],
        }
        assert Document(adf).to_markdown() == "Hello, **world!**"

    def test_italic(self):
        adf = {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello, "},
                {"type": "text", "text": "world!", "marks": [{"type": "em"}]},
            ],
        }
        assert Document(adf).to_markdown() == "Hello, *world!*"

    def test_bold_italic(self):
        adf = {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello!", "marks": [{"type": "strong"}, {"type": "em"}]},
            ],
        }
        assert Document(adf).to_markdown() == "***Hello!***"

    def test_link_shown_by_default(self):
        adf = {
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": "This is a link",
                    "marks": [{"type": "link", "attrs": {"href": "http://example.com/"}}],
                },
            ],
        }
        assert Document(adf).to_markdown() == "[This is a link](http://example.com/)"

    def test_link_hidden_when_disabled(self):
        adf = {
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": "This is a link",
                    "marks": [{"type": "link", "attrs": {"href": "http://example.com/"}}],
                },
            ],
        }
        config = MarkdownConfig(show_links=False)
        assert Document(adf).to_markdown(config) == "[This is a link]"


class TestHeadings:
    def test_h1(self):
        adf = {
            "type": "heading",
            "attrs": {"level": 1},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        assert Document(adf).to_markdown() == "# My Heading"

    def test_h2(self):
        adf = {
            "type": "heading",
            "attrs": {"level": 2},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        assert Document(adf).to_markdown() == "## My Heading"

    def test_h6(self):
        adf = {
            "type": "heading",
            "attrs": {"level": 6},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        assert Document(adf).to_markdown() == "###### My Heading"


class TestLists:
    def test_bullet_list(self):
        adf = {
            "type": "bulletList",
            "content": [
                {
                    "type": "listItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Item 1"}]}
                    ],
                },
                {
                    "type": "listItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Item 2"}]}
                    ],
                },
            ],
        }
        assert Document(adf).to_markdown() == "- Item 1\n- Item 2"

    def test_ordered_list(self):
        adf = {
            "type": "orderedList",
            "content": [
                {
                    "type": "listItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "First"}]}
                    ],
                },
                {
                    "type": "listItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Second"}]}
                    ],
                },
            ],
        }
        assert Document(adf).to_markdown() == "1. First\n2. Second"

    def test_task_list(self):
        adf = {
            "type": "taskList",
            "content": [
                {
                    "type": "taskItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Task 1"}]}
                    ],
                },
            ],
        }
        assert Document(adf).to_markdown() == "- [ ] Task 1"


class TestCodeBlocks:
    def test_with_language(self):
        adf = {
            "type": "codeBlock",
            "attrs": {"language": "python"},
            "content": [{"type": "text", "text": "print('hello')"}],
        }
        assert Document(adf).to_markdown() == "```python\nprint('hello')\n```"

    def test_without_language(self):
        adf = {"type": "codeBlock", "content": [{"type": "text", "text": "some code"}]}
        assert Document(adf).to_markdown() == "```\nsome code\n```"


class TestBlockElements:
    def test_blockquote(self):
        adf = {
            "type": "blockquote",
            "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Quote text"}]}],
        }
        assert Document(adf).to_markdown() == "> Quote text"

    def test_blockquote_two_paragraphs(self):
        adf = {
            "type": "blockquote",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "X"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "Y"}]},
            ],
        }
        result = Document(adf).to_markdown()
        # Paragraphs must be separated by a blank quoted line
        assert result == "> X\n>\n> Y"

    def test_panel(self):
        adf = {
            "type": "panel",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Panel content"}]}
            ],
        }
        assert Document(adf).to_markdown() == "> Panel content"

    def test_panel_two_paragraphs(self):
        adf = {
            "type": "panel",
            "attrs": {"panelType": "info"},
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "A"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": "B"}]},
            ],
        }
        result = Document(adf).to_markdown()
        # Paragraphs must be separated by a blank quoted line
        assert result == "> A\n>\n> B"

    def test_table_cell_two_paragraphs(self):
        adf = {
            "type": "table",
            "content": [
                {
                    "type": "tableRow",
                    "content": [
                        {
                            "type": "tableCell",
                            "content": [
                                {"type": "paragraph", "content": [{"type": "text", "text": "P1"}]},
                                {"type": "paragraph", "content": [{"type": "text", "text": "P2"}]},
                            ],
                        }
                    ],
                }
            ],
        }
        result = Document(adf).to_markdown()
        assert "P1" in result
        assert "P2" in result


class TestStatus:
    def test_status_badge(self):
        adf = {"type": "status", "attrs": {"text": "DONE", "color": "green"}}
        assert Document(adf).to_markdown() == "**[DONE]**"


class TestEmoji:
    def test_with_text(self):
        adf = {"type": "emoji", "attrs": {"shortName": ":grinning:", "text": "😀"}}
        assert Document(adf).to_markdown() == "😀"

    def test_fallback_to_shortname(self):
        adf = {"type": "emoji", "attrs": {"shortName": ":thumbsup:"}}
        assert Document(adf).to_markdown() == ":thumbsup:"

    def test_in_paragraph(self):
        adf = {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello "},
                {"type": "emoji", "attrs": {"shortName": ":wave:", "text": "👋"}},
                {"type": "text", "text": " world!"},
            ],
        }
        assert Document(adf).to_markdown() == "Hello 👋 world!"

    def test_atlassian_custom(self):
        adf = {
            "type": "emoji",
            "attrs": {"shortName": ":awthanks:", "id": "atlassian-awthanks", "text": ":awthanks:"},
        }
        assert Document(adf).to_markdown() == ":awthanks:"


class TestMention:
    def test_mention(self):
        adf = {
            "type": "mention",
            "attrs": {"id": "8675309", "text": "@Tommy Tutone", "accessLevel": ""},
        }
        assert Document(adf).to_markdown() == "@Tommy Tutone"


class TestBlockCard:
    def test_with_url(self):
        adf = {
            "type": "blockCard",
            "attrs": {"url": "http://example.com"},
        }
        assert Document(adf).to_markdown() == "[http://example.com]"

    def test_with_data(self):
        adf = {
            "type": "blockCard",
            "attrs": {"data": '{"title":"Example"}'},
        }
        assert Document(adf).to_markdown() == '```\n{"title":"Example"}\n```'

    def test_broken_block_card(self):
        adf = {
            "type": "blockCard",
            "attrs": {},
        }
        assert Document(adf).to_markdown() == "<broken_blockcard>"

    def test_in_document(self):
        adf = {
            "type": "doc",
            "content": [
                {"type": "paragraph"},
                {
                    "type": "blockCard",
                    "attrs": {
                        "url": "https://gitlab.com/redhat/centos-stream/src/kernel/centos-stream-9/-/merge_requests/7939"
                    },
                },
            ],
        }
        assert Document(adf).to_markdown() == (
            "[https://gitlab.com/redhat/centos-stream/src/kernel/centos-stream-9/-/merge_requests/7939]"
        )


class TestKnownUnsupportedNodes:
    def test_extension_warns_by_default(self):
        adf = {
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Before"}]},
                {
                    "type": "extension",
                    "attrs": {
                        "extensionType": "com.atlassian.confluence.macro.core",
                        "extensionKey": "toc",
                    },
                },
                {"type": "paragraph", "content": [{"type": "text", "text": "After"}]},
            ],
        }

        with pytest.warns(UserWarning, match='Known unsupported node type "extension"'):
            assert Document(adf).to_markdown(on_known_unsupported="warn") == "Before\n\nAfter"

    def test_extension_can_error(self):
        with pytest.raises(UnsupportedNodeTypeError):
            Document({"type": "extension"}).to_markdown(on_known_unsupported="error")

    def test_extension_can_warn(self):
        doc = Document({"type": "extension"})

        with pytest.warns(UserWarning, match='Known unsupported node type "extension"'):
            assert doc.to_markdown(on_known_unsupported="warn") == ""

    def test_extension_can_render_as_html(self):
        adf = {
            "type": "extension",
            "attrs": {
                "extensionKey": "toc",
                "extensionType": "com.atlassian.confluence.macro.core",
            },
        }

        assert Document(adf).to_markdown(on_known_unsupported="html") == (
            '<div adf="extension" '
            'params=\'{"extensionKey":"toc","extensionType":"com.atlassian.confluence.macro.core"}\'></div>'
        )

    def test_extension_in_table_cell_renders_as_span(self):
        adf = {
            "type": "table",
            "content": [
                {
                    "type": "tableRow",
                    "content": [
                        {
                            "type": "tableCell",
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Before "}],
                                },
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
            ],
        }

        assert Document(adf).to_markdown(on_known_unsupported="html") == (
            '| Before <span adf="extension" '
            'params=\'{"extensionKey":"toc",'
            '"extensionType":"com.atlassian.confluence.macro.core"}\'></span> |'
        )
