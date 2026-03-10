"""Tests for error handling and error messages."""

from typing import cast

import pytest
from pyadf import (
    Document,
    InvalidFieldError,
    InvalidInputError,
    InvalidJSONError,
    MissingFieldError,
    UnsupportedNodeTypeError,
)
from pyadf._types import JSONObject


class TestJSONErrors:
    """Test JSON parsing errors."""

    def test_invalid_json_string(self):
        """Test error message for invalid JSON."""
        with pytest.raises(InvalidJSONError) as exc_info:
            Document('{"type": "doc", invalid}')

        error = exc_info.value
        assert "Invalid JSON" in str(error)
        assert error.json_error is not None

    def test_invalid_input_type(self):
        """Test error message for invalid input type."""
        with pytest.raises(InvalidInputError) as exc_info:
            Document(cast("str | JSONObject | None", 12345))

        error = exc_info.value
        assert "Invalid input type" in str(error)
        assert "expected str, dict, or None" in str(error)
        assert "got int" in str(error)
        assert "Hint:" in str(error)

    def test_json_must_decode_to_object(self):
        """Test that JSON strings decode to an object node."""
        with pytest.raises(InvalidInputError) as exc_info:
            Document('[{"type": "paragraph"}]')

        error = exc_info.value
        assert "expected JSON object, dict, or None" in str(error)
        assert "got list" in str(error)


class TestMissingFieldErrors:
    """Test missing field errors."""

    def test_missing_type_field(self):
        """Test error message when 'type' field is missing."""
        with pytest.raises(MissingFieldError) as exc_info:
            Document({"content": []})

        error = exc_info.value
        assert 'Missing required field "type"' in str(error)
        assert "at: <root>" in str(error)
        assert "Expected one of:" in str(error)
        # Should show some supported types
        assert "doc" in str(error) or "paragraph" in str(error)

    def test_missing_type_in_nested_node(self):
        """Test error message with node path for nested nodes."""
        with pytest.raises(MissingFieldError) as exc_info:
            Document(
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [
                                {"text": "missing type field"}  # Missing 'type' field
                            ],
                        }
                    ],
                }
            )

        error = exc_info.value
        assert 'Missing required field "type"' in str(error)
        # Should show the path to the problematic node
        assert "paragraph" in str(error).lower()


class TestUnsupportedNodeTypes:
    """Test unsupported node type errors."""

    def test_unsupported_node_type(self):
        """Test error message for unsupported node types."""
        with pytest.raises(UnsupportedNodeTypeError) as exc_info:
            Document({"type": "foobar"})

        error = exc_info.value
        assert 'Unsupported node type "foobar"' in str(error)
        assert "Supported types:" in str(error)
        # Should list some supported types
        assert "doc" in str(error) or "paragraph" in str(error)

    def test_unsupported_nested_node_type(self):
        """Test error with path for unsupported nested node."""
        with pytest.raises(UnsupportedNodeTypeError) as exc_info:
            Document(
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [{"type": "invalidNodeType", "text": "test"}],
                        }
                    ],
                }
            )

        error = exc_info.value
        assert "invalidNodeType" in str(error)
        assert "at:" in str(error)
        # Path should show where the error occurred
        assert "paragraph" in str(error).lower()


class TestErrorMessageQuality:
    """Test that error messages are helpful and clear."""

    def test_error_has_node_path(self):
        """Test that errors include node path for context."""
        with pytest.raises(UnsupportedNodeTypeError) as exc_info:
            Document(
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "bulletList",
                            "content": [
                                {
                                    "type": "listItem",
                                    "content": [
                                        {
                                            "type": "unknownType",
                                            "content": [],
                                        }
                                    ],
                                }
                            ],
                        }
                    ],
                }
            )

        error = exc_info.value
        error_str = str(error)

        # Should show the full path
        assert "at:" in error_str
        # Path should include parent nodes
        assert "bulletList" in error_str or "listItem" in error_str

    def test_invalid_field_shows_valid_options(self):
        """Test that invalid field errors show valid options."""
        with pytest.raises(UnsupportedNodeTypeError) as exc_info:
            Document({"type": "notAValidType"})

        error = exc_info.value
        error_str = str(error)

        # Should list valid types
        assert "Supported types:" in error_str
        # Should show actual supported node types
        assert '"doc"' in error_str or '"paragraph"' in error_str


class TestValidData:
    """Test that valid data doesn't raise errors."""

    def test_valid_document(self):
        """Test that valid documents don't raise errors."""
        # This should not raise any errors
        doc = Document(
            {
                "type": "doc",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [{"type": "text", "text": "Hello, world!"}],
                    }
                ],
            }
        )
        result = doc.to_markdown()
        assert result == "Hello, world!"

    def test_empty_document(self):
        """Test that empty documents work correctly."""
        doc = Document()
        result = doc.to_markdown()
        assert result == ""

    def test_none_document(self):
        """Test that None input works correctly."""
        doc = Document(None)
        result = doc.to_markdown()
        assert result == ""


class TestMentionFallback:
    """Test mention rendering fallbacks."""

    def test_mention_without_text_returns_empty_string(self):
        """Test mention nodes without text render as empty string."""
        result = Document({"type": "mention", "attrs": {"id": "8675309"}}).to_markdown()
        assert result == ""
