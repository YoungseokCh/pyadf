"""Tests for error handling, error messages, and validation."""

import pytest

from pyadf import (
    Document,
    InvalidFieldError,
    InvalidInputError,
    InvalidJSONError,
    MissingFieldError,
    UnsupportedNodeTypeError,
)


class TestJSONErrors:
    def test_invalid_json_string(self):
        with pytest.raises(InvalidJSONError) as exc_info:
            Document('{"type": "doc", invalid}')
        error = exc_info.value
        assert "Invalid JSON" in str(error)
        assert error.json_error is not None

    def test_invalid_input_type(self):
        with pytest.raises(InvalidInputError) as exc_info:
            Document(12345)
        error = exc_info.value
        assert "Invalid input type" in str(error)
        assert "expected str, dict, or None" in str(error)
        assert "got int" in str(error)
        assert "Hint:" in str(error)


class TestJSONErrorPosition:
    def test_single_line(self):
        bad_json = '{"type": "doc", invalid}'
        with pytest.raises(InvalidJSONError) as exc_info:
            Document(bad_json)
        error = exc_info.value
        assert error.position is not None
        assert error.position >= 16

    def test_multiline_absolute_offset(self):
        """Position should be an absolute byte offset, not line-local column."""
        bad_json = "{\n    XXXXX}"
        with pytest.raises(InvalidJSONError) as exc_info:
            Document(bad_json)
        error = exc_info.value
        assert error.position is not None
        # "XXXXX" starts at byte 6 (after '{\n    '). Line-local would be 4.
        assert error.position >= 6


class TestMissingFieldErrors:
    def test_missing_type_at_root(self):
        with pytest.raises(MissingFieldError) as exc_info:
            Document({"content": []})
        error = exc_info.value
        assert 'Missing required field "type"' in str(error)
        assert "at: <root>" in str(error)
        assert "Expected one of:" in str(error)
        assert "doc" in str(error) or "paragraph" in str(error)

    def test_missing_type_in_nested_node(self):
        with pytest.raises(MissingFieldError) as exc_info:
            Document(
                {
                    "type": "doc",
                    "content": [
                        {
                            "type": "paragraph",
                            "content": [{"text": "missing type field"}],
                        }
                    ],
                }
            )
        error = exc_info.value
        assert 'Missing required field "type"' in str(error)
        assert "paragraph" in str(error).lower()


class TestUnsupportedNodeTypes:
    def test_unsupported_at_root(self):
        with pytest.raises(UnsupportedNodeTypeError) as exc_info:
            Document({"type": "foobar"})
        error = exc_info.value
        assert 'Unsupported node type "foobar"' in str(error)
        assert "Supported types:" in str(error)
        assert "doc" in str(error) or "paragraph" in str(error)

    def test_unsupported_nested(self):
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
        assert "paragraph" in str(error).lower()


class TestInvalidFieldErrors:
    def test_invalid_content_field(self):
        """content must be a list of dicts, not a string."""
        with pytest.raises(InvalidFieldError) as exc_info:
            Document({"type": "doc", "content": "bad"})
        assert "content" in str(exc_info.value)

    def test_invalid_attrs_field(self):
        """attrs must be a dict, not a string."""
        with pytest.raises(InvalidFieldError) as exc_info:
            Document({"type": "doc", "attrs": "bad"})
        assert "attrs" in str(exc_info.value)

    def test_invalid_marks_field(self):
        """marks must be a list of dicts."""
        with pytest.raises(InvalidFieldError) as exc_info:
            Document({"type": "text", "text": "x", "marks": "bad"})
        assert "marks" in str(exc_info.value)


class TestErrorMessageQuality:
    def test_error_has_node_path(self):
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
                                    "content": [{"type": "unknownType", "content": []}],
                                }
                            ],
                        }
                    ],
                }
            )
        error_str = str(exc_info.value)
        assert "at:" in error_str
        assert "bulletList" in error_str or "listItem" in error_str

    def test_shows_valid_options(self):
        with pytest.raises(UnsupportedNodeTypeError) as exc_info:
            Document({"type": "notAValidType"})
        error_str = str(exc_info.value)
        assert "Supported types:" in error_str
        assert '"doc"' in error_str or '"paragraph"' in error_str


class TestRecursionDepth:
    def test_deeply_nested_adf_raises(self):
        """ADF nesting beyond 200 levels should raise an error, not stack overflow."""
        node = {"type": "text", "text": "deep"}
        for _ in range(300):
            node = {"type": "doc", "content": [node]}
        with pytest.raises(InvalidInputError) as exc_info:
            Document(node)
        assert "depth" in str(exc_info.value)


class TestValidData:
    def test_valid_document(self):
        doc = Document(
            {
                "type": "doc",
                "content": [
                    {"type": "paragraph", "content": [{"type": "text", "text": "Hello, world!"}]},
                ],
            }
        )
        assert doc.to_markdown() == "Hello, world!"
