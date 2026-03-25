"""Tests for the JSONL streaming API (convert_jsonl)."""

import io
import json

import pytest

from pyadf import ConversionError, MarkdownConfig, PyADFError, convert_jsonl
from tests._helpers import make_adf_json, make_jsonl


class TestHappyPath:
    def test_basic_conversion(self):
        lines = [make_adf_json("Hello"), make_adf_json("World")]
        assert list(convert_jsonl(make_jsonl(lines))) == ["Hello", "World"]

    def test_single_document(self):
        data = make_adf_json("Solo").encode() + b"\n"
        assert list(convert_jsonl(data)) == ["Solo"]

    def test_empty_input(self):
        assert list(convert_jsonl(b"")) == []

    def test_with_config(self):
        doc = json.dumps(
            {
                "type": "bulletList",
                "content": [
                    {
                        "type": "listItem",
                        "content": [
                            {"type": "paragraph", "content": [{"type": "text", "text": "Item"}]}
                        ],
                    }
                ],
            }
        )
        config = MarkdownConfig(bullet_marker="*")
        assert list(convert_jsonl(doc.encode() + b"\n", config=config)) == ["* Item"]

    def test_100_documents(self):
        docs = [make_adf_json(f"Doc {i}") for i in range(100)]
        results = list(convert_jsonl(make_jsonl(docs)))
        assert len(results) == 100
        assert results[0] == "Doc 0"
        assert results[99] == "Doc 99"

    def test_from_file_path(self, tmp_path):
        data = make_jsonl([make_adf_json("FromFile")])
        path = tmp_path / "test.jsonl"
        path.write_bytes(data)
        assert list(convert_jsonl(str(path))) == ["FromFile"]

    def test_from_binary_io(self):
        data = make_jsonl([make_adf_json("FromIO")])
        assert list(convert_jsonl(io.BytesIO(data))) == ["FromIO"]


class TestErrorHandling:
    def test_include_errors_default(self):
        lines = [make_adf_json("Good"), "not json", make_adf_json("AlsoGood")]
        results = list(convert_jsonl(make_jsonl(lines)))
        assert results[0] == "Good"
        assert isinstance(results[1], ConversionError)
        assert results[1].line_number == 2
        assert "not json" in results[1].raw_line
        assert results[2] == "AlsoGood"

    def test_raise_on_error(self):
        lines = [make_adf_json("OK"), "bad json"]
        with pytest.raises(PyADFError):
            list(convert_jsonl(make_jsonl(lines), on_error="raise"))

    def test_skip_errors(self):
        lines = [make_adf_json("A"), "bad", make_adf_json("B")]
        assert list(convert_jsonl(make_jsonl(lines), on_error="skip")) == ["A", "B"]

    def test_invalid_adf_structure(self):
        lines = [make_adf_json("OK"), '{"type":"totallyFake"}']
        results = list(convert_jsonl(make_jsonl(lines), on_error="include"))
        assert isinstance(results[0], str)
        assert isinstance(results[1], ConversionError)
        assert "totallyFake" in results[1].error


class TestBatching:
    def test_small_batch_size(self):
        docs = [make_adf_json(f"D{i}") for i in range(10)]
        results = list(convert_jsonl(make_jsonl(docs), batch_size=3))
        assert len(results) == 10
        assert results[0] == "D0"
        assert results[9] == "D9"

    def test_batch_size_1(self):
        docs = [make_adf_json("X"), make_adf_json("Y")]
        assert list(convert_jsonl(make_jsonl(docs), batch_size=1)) == ["X", "Y"]


class TestBlankLines:
    def test_blank_line_between_docs(self):
        """Blank lines should not shift error line numbers."""
        data = (make_adf_json("A") + "\n\nnot json\n").encode()
        results = list(convert_jsonl(data, on_error="include"))
        assert results[0] == "A"
        err = results[1]
        assert isinstance(err, ConversionError)
        assert err.line_number == 3
        assert "not json" in err.raw_line

    def test_multiple_blank_lines(self):
        data = (make_adf_json("A") + "\n\n\n\n" + make_adf_json("B") + "\n").encode()
        assert list(convert_jsonl(data)) == ["A", "B"]

    def test_crlf_lines(self):
        data = (make_adf_json("A") + "\r\n" + make_adf_json("B") + "\r\n").encode()
        assert list(convert_jsonl(data)) == ["A", "B"]

    def test_trailing_newline(self):
        data = make_adf_json("A").encode() + b"\n"
        assert list(convert_jsonl(data)) == ["A"]

    def test_no_trailing_newline(self):
        data = make_adf_json("A").encode()
        assert list(convert_jsonl(data)) == ["A"]

    def test_multiple_trailing_newlines(self):
        data = make_adf_json("A").encode() + b"\n\n\n"
        assert list(convert_jsonl(data)) == ["A"]


class TestArgValidation:
    def test_batch_size_zero_raises(self):
        with pytest.raises(ValueError, match="batch_size"):
            list(convert_jsonl(b"", batch_size=0))

    def test_batch_size_negative_raises(self):
        with pytest.raises(ValueError, match="batch_size"):
            list(convert_jsonl(b"", batch_size=-1))

    def test_invalid_on_error_raises(self):
        with pytest.raises(ValueError, match="on_error"):
            list(convert_jsonl(b"", on_error="invalid"))  # type: ignore[arg-type]
