"""Basic tests for pyadf functionality."""

import pytest
from pyadf import adf2md


class TestSimpleConversions:
    """Test simple ADF to Markdown conversions."""

    def test_simple_paragraph(self):
        """Test converting a simple paragraph."""
        adf_data = {
            "type": "paragraph",
            "content": [{"type": "text", "text": "Hello, world!"}],
        }
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
        assert result == "# My Heading"

    def test_heading_level_2(self):
        """Test converting h2 heading."""
        adf_data = {
            "type": "heading",
            "attrs": {"level": 2},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        result = adf2md(adf_data)
        assert result == "## My Heading"

    def test_heading_level_6(self):
        """Test converting h6 heading."""
        adf_data = {
            "type": "heading",
            "attrs": {"level": 6},
            "content": [{"type": "text", "text": "My Heading"}],
        }
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
        assert result == "```python\nprint('hello')\n```"

    def test_code_block_without_language(self):
        """Test converting code block without language."""
        adf_data = {
            "type": "codeBlock",
            "content": [{"type": "text", "text": "some code"}],
        }
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
        assert result == "> Quote text"

    def test_panel(self):
        """Test converting panel."""
        adf_data = {
            "type": "panel",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Panel content"}]}
            ],
        }
        result = adf2md(adf_data)
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
        result = adf2md(adf_data)
        assert "First paragraph" in result
        assert "Second paragraph" in result

    def test_none_input(self):
        """Test converting None input."""
        result = adf2md(None)
        assert result == ""

    def test_list_input(self):
        """Test converting list of nodes."""
        adf_data = [
            {"type": "paragraph", "content": [{"type": "text", "text": "First"}]},
            {"type": "paragraph", "content": [{"type": "text", "text": "Second"}]},
        ]
        result = adf2md(adf_data)
        assert "First" in result
        assert "Second" in result


class TestStatus:
    """Test status badge conversions."""

    def test_status_badge(self):
        """Test converting status badge."""
        adf_data = {
            "type": "status",
            "attrs": {"text": "DONE", "color": "green"},
        }
        result = adf2md(adf_data)
        assert result == "**[DONE]**"
