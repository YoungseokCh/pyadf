"""Basic tests for pyadf functionality."""

import pytest
from pyadf import Document, MarkdownConfig


class TestSimpleConversions:
    """Test simple ADF to Markdown conversions."""

    def test_simple_paragraph(self):
        """Test converting a simple paragraph."""
        adf_data = {
            "type": "doc",
            "content": [
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Hello, world!"}],
                }
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "Hello, world!"

    def test_bold_text(self):
        """Test converting bold text."""
        adf_data = {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello, "},
                {"type": "text", "text": "world!", "marks": [{"type": "strong"}]},
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "Hello, **world!**"

    def test_italic_text(self):
        """Test converting italic text."""
        adf_data = {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello, "},
                {"type": "text", "text": "world!", "marks": [{"type": "em"}]},
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "Hello, *world!*"

    def test_bold_italic_text(self):
        """Test converting bold and italic text."""
        adf_data = {
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": "Hello!",
                    "marks": [{"type": "strong"}, {"type": "em"}],
                }
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "***Hello!***"


class TestHeadings:
    """Test heading conversions."""

    def test_heading_level_1(self):
        """Test converting h1 heading."""
        adf_data = {
            "type": "heading",
            "attrs": {"level": 1},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        result = Document(adf_data).to_markdown()
        assert result == "# My Heading"

    def test_heading_level_2(self):
        """Test converting h2 heading."""
        adf_data = {
            "type": "heading",
            "attrs": {"level": 2},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        result = Document(adf_data).to_markdown()
        assert result == "## My Heading"

    def test_heading_level_6(self):
        """Test converting h6 heading."""
        adf_data = {
            "type": "heading",
            "attrs": {"level": 6},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        result = Document(adf_data).to_markdown()
        assert result == "###### My Heading"


class TestLists:
    """Test list conversions."""

    def test_bullet_list(self):
        """Test converting a bullet list."""
        adf_data = {
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
        result = Document(adf_data).to_markdown()
        assert result == "+ Item 1\n+ Item 2"

    def test_ordered_list(self):
        """Test converting an ordered list."""
        adf_data = {
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
        result = Document(adf_data).to_markdown()
        assert result == "1. First\n2. Second"

    def test_task_list(self):
        """Test converting a task list."""
        adf_data = {
            "type": "taskList",
            "content": [
                {
                    "type": "taskItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Task 1"}]}
                    ],
                }
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "- [ ] Task 1"


class TestCodeBlocks:
    """Test code block conversions."""

    def test_code_block_with_language(self):
        """Test converting code block with language."""
        adf_data = {
            "type": "codeBlock",
            "attrs": {"language": "python"},
            "content": [{"type": "text", "text": "print('hello')"}],
        }
        result = Document(adf_data).to_markdown()
        assert result == "```python\nprint('hello')\n```"

    def test_code_block_without_language(self):
        """Test converting code block without language."""
        adf_data = {
            "type": "codeBlock",
            "content": [{"type": "text", "text": "some code"}],
        }
        result = Document(adf_data).to_markdown()
        assert result == "```\nsome code\n```"


class TestBlockElements:
    """Test block element conversions."""

    def test_blockquote(self):
        """Test converting blockquote."""
        adf_data = {
            "type": "blockquote",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Quote text"}]}
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "> Quote text"

    def test_panel(self):
        """Test converting panel."""
        adf_data = {
            "type": "panel",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Panel content"}]}
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "> Panel content"


class TestDocument:
    """Test full document conversions."""

    def test_document(self):
        """Test converting a full document."""
        adf_data = {
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "First paragraph"}]},
                {
                    "type": "paragraph",
                    "content": [{"type": "text", "text": "Second paragraph"}],
                },
            ],
        }
        result = Document(adf_data).to_markdown()
        assert "First paragraph" in result
        assert "Second paragraph" in result


class TestStatus:
    """Test status badge conversions."""

    def test_status_badge(self):
        """Test converting status badge."""
        adf_data = {
            "type": "status",
            "attrs": {"text": "DONE", "color": "green"},
        }
        result = Document(adf_data).to_markdown()
        assert result == "**[DONE]**"


class TestMarkdownConfig:
    """Test MarkdownConfig options."""

    def test_default_bullet_marker(self):
        """Test default bullet marker is +."""
        adf_data = {
            "type": "bulletList",
            "content": [
                {
                    "type": "listItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Item"}]}
                    ],
                },
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "+ Item"

    def test_asterisk_bullet_marker(self):
        """Test bullet marker can be set to *."""
        adf_data = {
            "type": "bulletList",
            "content": [
                {
                    "type": "listItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Item"}]}
                    ],
                },
            ],
        }
        config = MarkdownConfig(bullet_marker="*")
        result = Document(adf_data).to_markdown(config)
        assert result == "* Item"

    def test_dash_bullet_marker(self):
        """Test bullet marker can be set to -."""
        adf_data = {
            "type": "bulletList",
            "content": [
                {
                    "type": "listItem",
                    "content": [
                        {"type": "paragraph", "content": [{"type": "text", "text": "Item"}]}
                    ],
                },
            ],
        }
        config = MarkdownConfig(bullet_marker="-")
        result = Document(adf_data).to_markdown(config)
        assert result == "- Item"

    def test_invalid_bullet_marker(self):
        """Test that invalid bullet marker raises ValueError."""
        with pytest.raises(ValueError, match="Invalid bullet_marker"):
            MarkdownConfig(bullet_marker="x")


class TestEmoji:
    """Test emoji conversions."""

    def test_emoji_with_text(self):
        """Test emoji with text attribute returns unicode."""
        adf_data = {
            "type": "emoji",
            "attrs": {"shortName": ":grinning:", "text": "😀"},
        }
        result = Document(adf_data).to_markdown()
        assert result == "😀"

    def test_emoji_without_text_fallback_to_shortname(self):
        """Test emoji without text falls back to shortName."""
        adf_data = {
            "type": "emoji",
            "attrs": {"shortName": ":thumbsup:"},
        }
        result = Document(adf_data).to_markdown()
        assert result == ":thumbsup:"

    def test_emoji_in_paragraph(self):
        """Test emoji within a paragraph."""
        adf_data = {
            "type": "paragraph",
            "content": [
                {"type": "text", "text": "Hello "},
                {"type": "emoji", "attrs": {"shortName": ":wave:", "text": "👋"}},
                {"type": "text", "text": " world!"},
            ],
        }
        result = Document(adf_data).to_markdown()
        assert result == "Hello 👋 world!"

    def test_atlassian_emoji(self):
        """Test Atlassian custom emoji falls back to shortName."""
        adf_data = {
            "type": "emoji",
            "attrs": {
                "shortName": ":awthanks:",
                "id": "atlassian-awthanks",
                "text": ":awthanks:",
            },
        }
        result = Document(adf_data).to_markdown()
        assert result == ":awthanks:"
